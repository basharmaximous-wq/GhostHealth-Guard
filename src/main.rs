mod models;
mod github;

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Router, response::IntoResponse,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use secrecy::{SecretString, ExposeSecret};
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventType};
use octocrab::Octocrab;
use sqlx::PgPool;
use std::sync::Arc;
use anyhow::Context;

#[derive(Clone)]
pub struct AppState {
    webhook_secret: SecretString,
    app_id: u64,
    private_key: Vec<u8>,
    db: PgPool,
}

type HmacSha256 = Hmac<Sha256>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Logging (Sentinel Eyes)
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // 2. Load .env
    dotenvy::dotenv().ok();

    // 3. Setup Database
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    tracing::info!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await.context("Failed to connect to DB")?;

    // 4. Load rest of config
    let app_id: u64 = std::env::var("GITHUB_APP_ID")
        .context("GITHUB_APP_ID must be set")?
        .parse()
        .context("GITHUB_APP_ID must be a number")?;

    let webhook_secret = std::env::var("GITHUB_WEBHOOK_SECRET").context("GITHUB_WEBHOOK_SECRET must be set")?;
    
    let private_key_path = std::env::var("PRIVATE_KEY_PATH").unwrap_or_else(|_| "private-key.pem".to_string());
    let private_key = std::fs::read(private_key_path).context("private-key.pem not found")?;

    let state = Arc::new(AppState {
        webhook_secret: SecretString::new(webhook_secret),
        app_id,
        private_key,
        db: pool,
    });

    // 5. Final Server Setup
    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("👻 GhostHealth Guard active on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // SECURITY: Verify signature
    if let Err(err_resp) = verify_signature(&state, &headers, &body) {
        tracing::warn!("Blocked unauthorized webhook attempt");
        return err_resp;
    }

    // PARSING: Parse event
    let event_type = headers.get("X-GitHub-Event")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    let event = match WebhookEvent::try_from_header_and_body(event_type, &body) {
        Ok(ev) => ev,
        Err(e) => {
            tracing::error!("Payload parse error: {:?}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    // LOGIC: Process PR events in the background
    if let WebhookEventType::PullRequest = event.kind {
        tracing::info!("Pull Request event received. Spawning audit task...");
        tokio::spawn(async move {
            if let Err(e) = process_pr_event(state, event).await {
                tracing::error!("PR Processing Error: {:?}", e);
            }
        });
    }

    StatusCode::ACCEPTED.into_response() 
}

fn verify_signature(state: &AppState, headers: &HeaderMap, body: &Bytes) -> Result<(), axum::response::Response> {
    let signature = headers.get("X-Hub-Signature-256")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Missing signature").into_response())?;

    let remote_sha = signature.strip_prefix("sha256=")
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid signature format").into_response())?;

    let remote_sig_bytes = hex::decode(remote_sha)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid hex").into_response())?;

    let mut mac = HmacSha256::new_from_slice(state.webhook_secret.expose_secret().as_bytes())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;

    mac.update(body);
    let expected_sig = mac.finalize().into_bytes();

    if expected_sig.ct_eq(&remote_sig_bytes[..]).unwrap_u8() != 1 {
        return Err((StatusCode::UNAUTHORIZED, "Invalid signature").into_response());
    }
    Ok(())
}

async fn process_pr_event(state: Arc<AppState>, event: WebhookEvent) -> anyhow::Result<()> {
    let installation = event.installation.context("Missing installation info")?;
    let repo = event.repository.context("Missing repository info")?;
    let pr_event = event.content.as_pull_request().context("Not a PR event")?;
    
    let owner = &repo.owner.login;
    let name = &repo.name;
    let pr_number = pr_event.pull_request.number;

    tracing::info!("Starting audit for {}/{} PR #{}", owner, name, pr_number);

    let app_key = jsonwebtoken::EncodingKey::from_rsa_pem(&state.private_key)?;
    let octo = Octocrab::builder()
        .app(state.app_id.into(), app_key)
        .build()?;
    let client = octo.installation(installation.id);

    let context = github::get_pr_context(&client, owner, name, pr_number).await?;
    let report = github::run_privacy_audit(&context).await?;
    
    let has_violations = ["VIOLATION", "CRITICAL", "LEAK"].iter().any(|&word| report.to_uppercase().contains(word));
    let status = if has_violations { "VIOLATION" } else { "CLEAN" };

    sqlx::query!(
        "INSERT INTO audit_logs (repo_name, pr_number, status, report) VALUES ($1, $2, $3, $4)",
        name, pr_number as i32, status, report
    )
    .execute(&state.db)
    .await?;

    github::post_review(&client, owner, name, pr_number, &report, has_violations).await?;
    
    tracing::info!("Audit finished for PR #{}. Result: {}", pr_number, status);
    Ok(())
}
