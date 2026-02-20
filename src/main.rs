mod models;
mod github;

use axum::{body::Bytes, extract::State, http::{HeaderMap, StatusCode}, routing::post, Router, response::IntoResponse};
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
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();
    dotenvy::dotenv().ok();

    let pool = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
    let app_id: u64 = std::env::var("GITHUB_APP_ID")?.parse()?;
    let webhook_secret = std::env::var("GITHUB_WEBHOOK_SECRET")?;
    let private_key = std::fs::read(std::env::var("PRIVATE_KEY_PATH")?)?;

    let state = Arc::new(AppState {
        webhook_secret: SecretString::new(webhook_secret),
        app_id,
        private_key,
        db: pool,
    });

    let app = Router::new().route("/webhook", post(handle_webhook)).with_state(state);
    let addr = "0.0.0.0:3000";
    tracing::info!("🚀 GhostHealth Guard active on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

async fn handle_webhook(State(state): State<Arc<AppState>>, headers: HeaderMap, body: Bytes) -> impl IntoResponse {
    if let Err(e) = verify_signature(&state, &headers, &body) { return e; }

    let event_type = headers.get("X-GitHub-Event").and_then(|h| h.to_str().ok()).unwrap_or_default();
    let event = match WebhookEvent::try_from_header_and_body(event_type, &body) {
        Ok(ev) => ev,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    if let WebhookEventType::PullRequest = event.kind {
        tokio::spawn(async move {
            if let Err(e) = process_audit(state, event).await {
                tracing::error!("Audit Error: {:?}", e);
            }
        });
    }
    StatusCode::ACCEPTED.into_response()
}

fn verify_signature(state: &AppState, headers: &HeaderMap, body: &Bytes) -> Result<(), axum::response::Response> {
    let sig = headers.get("X-Hub-Signature-256").and_then(|h| h.to_str().ok())
        .ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;
    let remote_sha = sig.strip_prefix("sha256=").ok_or_else(|| StatusCode::UNAUTHORIZED.into_response())?;
    let mut mac = HmacSha256::new_from_slice(state.webhook_secret.expose_secret().as_bytes()).unwrap();
    mac.update(body);
    if mac.finalize().into_bytes().ct_eq(&hex::decode(remote_sha).unwrap()[..]).unwrap_u8() != 1 {
        return Err(StatusCode::UNAUTHORIZED.into_response());
    }
    Ok(())
}

async fn process_audit(state: Arc<AppState>, event: WebhookEvent) -> anyhow::Result<()> {
    let repo = event.repository.context("No repo")?;
    let pr = event.content.as_pull_request().context("No PR")?;
    let inst_id = event.installation.context("No installation")?.id;

    let app_key = jsonwebtoken::EncodingKey::from_rsa_pem(&state.private_key)?;
    let octo = Octocrab::builder().app(state.app_id.into(), app_key).build()?.installation(inst_id);

    let diff = github::get_pr_context(&octo, &repo.owner.login, &repo.name, pr.pull_request.number).await?;
    let report = github::run_privacy_audit(&diff).await?;
    
    let has_violations = report.to_uppercase().contains("VIOLATION");
    let status = if has_violations { "VIOLATION" } else { "CLEAN" };

    sqlx::query!("INSERT INTO audit_logs (repo_name, pr_number, status, report) VALUES ($1, $2, $3, $4)",
        repo.name, pr.pull_request.number as i32, status, report)
        .execute(&state.db).await?;

    github::post_review(&octo, &repo.owner.login, &repo.name, pr.pull_request.number, &report, has_violations).await?;
    Ok(())
}
