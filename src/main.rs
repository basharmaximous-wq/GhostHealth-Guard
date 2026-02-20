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
    dotenvy::dotenv().ok();

    // 1. Setup Database
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPool::connect(&database_url).await.context("Failed to connect to DB")?;
    
    // Optional: Run migrations
    // sqlx::migrate!().run(&pool).await?;

    // 2. Load Config
    let app_id: u64 = std::env::var("GITHUB_APP_ID")
        .context("GITHUB_APP_ID must be set")?
        .parse()
        .context("GITHUB_APP_ID must be a number")?;

    let webhook_secret = std::env::var("GITHUB_WEBHOOK_SECRET").context("GITHUB_WEBHOOK_SECRET must be set")?;
    let private_key = std::fs::read("private-key.pem").context("private-key.pem not found")?;

    let state = Arc::new(AppState {
        webhook_secret: SecretString::new(webhook_secret),
        app_id,
        private_key,
        db: pool,
    });

    // 3. Router
    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .with_state(state);

    // 4. Server Start
    let addr = "0.0.0.0:3000";
    println!("👻 GhostHealth Guard active on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // --- SECURITY: VERIFY SIGNATURE ---
    if let Err(err_resp) = verify_signature(&state, &headers, &body) {
        return err_resp;
    }

    // --- PARSING ---
    let event_type = headers.get("X-GitHub-Event")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    let event = match WebhookEvent::try_from_header_and_body(event_type, &body) {
        Ok(ev) => ev,
        Err(e) => {
            eprintln!("Payload parse error: {:?}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    // --- LOGIC: HANDLE PR EVENTS ---
    if let WebhookEventType::PullRequest = event.kind {
        // We spawn a task so we can return 200 OK to GitHub immediately (Best Practice)
        tokio::spawn(async move {
            if let Err(e) = process_pr_event(state, event).await {
                eprintln!("PR Processing Error: {:?}", e);
            }
        });
    }

    StatusCode::ACCEPTED.into_response() // 202 Accepted
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

    // Authenticate as the Installation
    let app_key = jsonwebtoken::EncodingKey::from_rsa_pem(&state.private_key)?;
    let octo = Octocrab::builder()
        .app(state.app_id.into(), app_key)
        .build()?;
    let client = octo.installation(installation.id);

    // Run Audit
    let context = github::get_pr_context(&client, owner, name, pr_number).await?;
    let report = github::run_privacy_audit(&context).await?;
    
    let has_violations = ["VIOLATION", "CRITICAL", "LEAK"].iter().any(|&word| report.to_uppercase().contains(word));
    let status = if has_violations { "VIOLATION" } else { "CLEAN" };

    // Update DB
    sqlx::query!(
        "INSERT INTO audit_logs (repo_name, pr_number, status, report) VALUES ($1, $2, $3, $4)",
        name, pr_number as i32, status, report
    )
    .execute(&state.db)
    .await?;

    // Post Professional Feedback
    github::post_review(&client, owner, name, pr_number, &report, has_violations).await?;

    Ok(())
}
