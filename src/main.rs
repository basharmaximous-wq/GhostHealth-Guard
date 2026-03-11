use dotenvy::dotenv;
use ghosthealth_guard::*;

use anyhow::Context;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use secrecy::SecretString;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load .env file
    dotenv().ok();

    // 2. FIPS Security Compliance
    fips::enable_fips().context("Failed to enable FIPS")?;
    fips::assert_fips_algorithm("AES-256-GCM");

    // 3. JSON Structured Logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .json()
        .init();

    info!("Starting GhostHealth-Guard: PostgreSQL Enabled");

    // 4. DATABASE
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/ghosthealth".to_string());

    let db = db::init_db(&database_url).await?;

    info!("Database connection and migrations completed");

    // 5. GitHub Configuration
    let webhook_secret = SecretString::new(
        std::env::var("GITHUB_WEBHOOK_SECRET").unwrap_or_else(|_| "local_test_secret".to_string()),
    );

    let github_app_id = std::env::var("GITHUB_APP_ID")
        .unwrap_or_else(|_| "0".to_string())
        .parse()
        .unwrap_or(0);

    let private_key_path =
        std::env::var("PRIVATE_KEY_PATH").unwrap_or_else(|_| "key.pem".to_string());

    let private_key = std::fs::read(&private_key_path).unwrap_or_else(|_| {
        tracing::warn!(
            "Private key not found at {}. Using empty key for testing.",
            private_key_path
        );
        vec![]
    });

    info!("GitHub App configuration loaded");

    let state = Arc::new(AppState {
        webhook_secret,
        github_app_id,
        private_key,
        db,
    });

    // 6. Routes
    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .route("/health", get(health))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    info!("LISTENING: Server started on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind to port 3000")?;

    axum::serve(listener, app).await.context("Server error")?;

    Ok(())
}

async fn health(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Health check heartbeat received");
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("Health check database error: {}", e);
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    if verify_webhook(&state, &headers, &body).is_err() {
        tracing::warn!("Webhook verification failed (Bypassing for local testing)");
    }

    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    if event_type != "pull_request" && event_type != "unknown" {
        tracing::info!("Ignoring non-PR event: {}", event_type);
        return StatusCode::ACCEPTED;
    }

    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to parse JSON: {:?}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = process_pull_request(state_clone, payload).await {
            tracing::error!("Failed to process PR: {:?}", e);
        }
    });

    tracing::info!("Queued PR processing job");
    StatusCode::ACCEPTED
}
