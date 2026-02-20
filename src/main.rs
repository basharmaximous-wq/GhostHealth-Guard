mod models;
mod github;

use ax_body::Bytes;
use axum::{
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

// 1. App State - The "Memory" of your bot
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
    // A. INITIALIZE LOGGING (The Sentinel's Eyes)
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // B. LOAD ENVIRONMENT
    dotenvy::dotenv().ok();
    tracing::info!("Initializing GhostHealth Guard...");

    // C. SETUP DATABASE
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPool::connect(&database_url)
        .await
        .context("Failed to connect to Postgres")?;
    
    // D. LOAD GITHUB CONFIG
    let app_id: u64 = std::env::var("GITHUB_APP_ID")?.parse()?;
    let webhook_secret = std::env::var("GITHUB_WEBHOOK_SECRET")?;
    let key_path = std::env::var("PRIVATE_KEY_PATH").unwrap_or_else(|_| "private-key.pem".to_string());
    let private_key = std::fs::read(key_path).context("Could not read private-key.pem")?;

    let state = Arc::new(AppState {
        webhook_secret: SecretString::new(webhook_secret),
        app_id,
        private_key,
        db: pool,
    });

    // E. DEFINE ROUTES
    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .with_state(state);

    // F. START SERVER
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    tracing::info!("🚀 GhostHealth Guard active on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// --- WEBHOOK HANDLER ---

async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // 1. Security Check
    if let Err(err_resp) = verify_signature(&state, &headers, &body) {
        tracing::warn!("Unauthorized webhook attempt blocked.");
        return err_resp;
    }

    // 2. Parse Event
    let event_type = headers.get("X-GitHub-Event").and_then(|h| h.to_str().ok()).unwrap_or_default();
    let event = match WebhookEvent::try_from_header_and_body(event_type, &body) {
        Ok(ev) => ev,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    // 3. Process Async (Don't make GitHub wait for the AI)
    if let WebhookEventType::PullRequest = event.kind {
        tracing::info!("Pull Request event received. Dispatching audit task...");
        tokio::spawn(async move {
            if let Err(e) = process_pr_event(state, event).await {
                tracing::error!("Audit Failed: {:?}", e);
            }
        });
    }

    StatusCode::ACCEPTED.into_response()
}

// --- HELPERS ---

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
    // ... (Your background logic here - Octocrab auth, AI audit, DB log) ...
    // Reference the logic we built in the previous step
    tracing::info!("PR Audit complete for repo: {}", event.repository.unwrap().name);
    Ok(())
}
