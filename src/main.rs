mod github;
mod scanner;
mod audit;
mod models;

use axum::{Router, routing::post, extract::State, body::Bytes, http::{HeaderMap, StatusCode}, response::IntoResponse};
use secrecy::{SecretString, ExposeSecret};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use sqlx::PgPool;
use std::sync::Arc;
use octocrab::{Octocrab, models::webhook_events::*};
use jsonwebtoken::{EncodingKey, Header};
use anyhow::Context;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
struct AppState {
    webhook_secret: SecretString,
    app_id: u64,
    private_key: Vec<u8>,
    db: PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fips::enable_fips();
    fips::assert_fips_algorithm("AES-256-GCM");

    let db = PgPool::connect(&std::env::var("DATABASE_URL")?).await?;

    let state = Arc::new(AppState {
        webhook_secret: SecretString::new(std::env::var("GITHUB_WEBHOOK_SECRET")?),
        app_id: std::env::var("GITHUB_APP_ID")?.parse()?,
        private_key: std::fs::read(std::env::var("PRIVATE_KEY_PATH")?)?,
        db,
    });

    let app = Router::new().route("/webhook", post(handle)).with_state(state);

    axum::serve(tokio::net::TcpListener::bind("0.0.0.0:3000").await?, app).await?;
    Ok(())
}

async fn handle(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {

    if verify(&state, &headers, &body).is_err() {
        return StatusCode::UNAUTHORIZED;
    }

    let event_type = headers.get("X-GitHub-Event")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let event = match WebhookEvent::try_from_header_and_body(event_type, &body) {
        Ok(e) => e,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    if let WebhookEventType::PullRequest = event.kind {
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = process_pr(state_clone, event).await {
                tracing::error!("Audit failed: {:?}", e);
            }
        });
    }

    StatusCode::ACCEPTED
}

async fn process_pr(state: Arc<AppState>, event: WebhookEvent) -> anyhow::Result<()> {

    let repo = event.repository.context("No repo")?;
    let pr_payload = match event.specific {
        WebhookEventPayload::PullRequest(p) => p,
        _ => return Ok(()),
    };

    let installation_id = match event.installation.context("No install")? {
        EventInstallation::Full(i) => i.id,
        EventInstallation::Minimal(i) => i.id,
    };

    let app_key = EncodingKey::from_rsa_pem(&state.private_key)?;
    let octo = Octocrab::builder()
        .app(state.app_id.into(), app_key)
        .build()?
        .installation(installation_id);

    let owner = repo.owner.unwrap().login;
    let repo_name = repo.name;
    let pr_number = pr_payload.pull_request.number;

    let diff = github::get_pr_diff(&octo, &owner, &repo_name, pr_number).await?;
    let result = github::process_diff(&diff).await?;

    sqlx::query("INSERT INTO audit_logs (repo_name, pr_number, status, risk_score, report) VALUES ($1,$2,$3,$4,$5)")
        .bind(&repo_name)
        .bind(pr_number as i32)
        .bind(&result.status)
        .bind(result.risk_score as i32)
        .bind(serde_json::to_value(&result)?)
        .execute(&state.db)
        .await?;

    github::post_review(&octo, &owner, &repo_name, pr_number, &result).await?;

    Ok(())
}

fn verify(state: &AppState, headers: &HeaderMap, body: &Bytes) -> Result<(), ()> {
    let sig = headers.get("X-Hub-Signature-256").ok_or(())?;
    let sig = sig.to_str().map_err(|_| ())?;
    let remote = sig.strip_prefix("sha256=").ok_or(())?;

    let mut mac = HmacSha256::new_from_slice(state.webhook_secret.expose_secret().as_bytes()).unwrap();
    mac.update(body);
    let local = mac.finalize().into_bytes();

    let remote = hex::decode(remote).map_err(|_| ())?;

    if local.ct_eq(&remote).unwrap_u8() != 1 {
        return Err(());
    }

    Ok(())
}
