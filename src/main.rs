mod models;
mod github;

use anyhow::Context;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use hmac::{Hmac, Mac};
use octocrab::models::webhook_events::{
    EventInstallation, WebhookEvent, WebhookEventPayload, WebhookEventType,
};
use octocrab::Octocrab;
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;
use sqlx::PgPool;
use std::sync::Arc;
use subtle::ConstantTimeEq;

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
   tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPool::connect(&database_url).await?;

    let app_id: u64 = std::env::var("GITHUB_APP_ID")?.parse()?;
    let webhook_secret = std::env::var("GITHUB_WEBHOOK_SECRET")?;
    let private_key_path =
        std::env::var("PRIVATE_KEY_PATH").unwrap_or_else(|_| "private-key.pem".to_string());
    let private_key = std::fs::read(private_key_path)?;

    let state = Arc::new(AppState {
        webhook_secret: SecretString::new(webhook_secret),
        app_id,
        private_key,
        db: pool,
    });

   let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .with_state(state);
    let addr = "0.0.0.0:3000";
    tracing::info!("🚀 GhostHealth Guard active on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    if verify_signature(&state, &headers, &body).is_err() {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();
    let event = match WebhookEvent::try_from_header_and_body(event_type, &body) {
        Ok(ev) => ev,
        Err(e) => {
            tracing::error!("Webhook parse error: {:?}", e);
            return StatusCode::BAD_REQUEST.into_response();
        }
    };

    if let WebhookEventType::PullRequest = event.kind {
        tokio::spawn(async move {
            if let Err(e) = process_audit(state, event).await {
                tracing::error!("Audit task failed: {:?}", e);
            }
        });
    }
    StatusCode::ACCEPTED.into_response()
}

fn verify_signature(state: &AppState, headers: &HeaderMap, body: &Bytes) -> Result<(), StatusCode> {
   let sig = headers
        .get("X-Hub-Signature-256")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let remote_sha = sig
        .strip_prefix("sha256=")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let mut mac =
        HmacSha256::new_from_slice(state.webhook_secret.expose_secret().as_bytes()).unwrap();
    mac.update(body);
    let remote_bytes = hex::decode(remote_sha).map_err(|_| StatusCode::UNAUTHORIZED)?;
    if mac
        .finalize()
        .into_bytes()
        .ct_eq(&remote_bytes[..])
        .unwrap_u8()
        != 1
    {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(())
}

async fn process_audit(state: Arc<AppState>, event: WebhookEvent) -> anyhow::Result<()> {
    let repo = event.repository.context("No repo found in webhook")?;

    // Octocrab 0.38: payload is in WebhookEventPayload
    let pr_content = match event.specific {
        WebhookEventPayload::PullRequest(payload) => payload,
        _ => return Err(anyhow::anyhow!("Not a PR event")),
    };

    let inst_id = match event.installation.context("No installation info")? {
        EventInstallation::Full(installation) => installation.id,
        EventInstallation::Minimal(installation) => installation.id,
    };
    let app_key = jsonwebtoken::EncodingKey::from_rsa_pem(&state.private_key)?;
    let octo = Octocrab::builder()
        .app(state.app_id.into(), app_key)
        .build()?
        .installation(inst_id);
    let pr_number = pr_content.pull_request.number;
    let owner = repo.owner.clone().context("No owner info")?.login;
    let repo_name = &repo.name;

    let diff = github::get_pr_context(&octo, &owner, repo_name, pr_number).await?;
    let report = github::run_privacy_audit(&diff).await?;

    let has_violations = report.to_uppercase().contains("VIOLATION");
    let status = if has_violations { "VIOLATION" } else { "CLEAN" };

    sqlx::query(
        "INSERT INTO audit_logs (repo_name, pr_number, status, report) VALUES ($1, $2, $3, $4)",
   )
        .bind(repo_name)
    .bind(pr_number as i32)
    .bind(status)
    .bind(&report)
    .execute(&state.db)
    .await?;

    github::post_review(&octo, &owner, repo_name, pr_number, &report, has_violations).await?;
    Ok(())
}
