use sqlx::PgPool;
mod fips;
mod github;
mod models;
mod scanner;
mod audit;

use anyhow::Context;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use hmac::{Hmac, Mac};
use jsonwebtoken::EncodingKey;
use octocrab::{models::webhook_events::*, Octocrab};
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tracing::info;

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
    // FIPS compliance first
    fips::enable_fips();
    fips::assert_fips_algorithm("AES-256-GCM");

    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .json()
        .init();

    info!("Starting GitHub webhook processor");

    // Database connection with proper pooling
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?)
        .await
        .context("Failed to connect to database")?;

    info!("Database connection established");

    // Load required environment variables
    let webhook_secret = SecretString::new(
        std::env::var("GITHUB_WEBHOOK_SECRET")
            .context("GITHUB_WEBHOOK_SECRET must be set")?
    );
    
    let app_id = std::env::var("GITHUB_APP_ID")
        .context("GITHUB_APP_ID must be set")?
        .parse()
        .context("GITHUB_APP_ID must be a valid u64")?;
    
    let private_key_path = std::env::var("PRIVATE_KEY_PATH")
        .context("PRIVATE_KEY_PATH must be set")?;
    
    let private_key = std::fs::read(&private_key_path)
        .with_context(|| format!("Failed to read private key from {}", private_key_path))?;

    info!("GitHub App configuration loaded");

    let state = Arc::new(AppState {
        webhook_secret,
        app_id,
        private_key,
        db,
    });

    // Build router with webhook and health endpoints
    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .route("/health", get(health))
        .with_state(state);

    let addr = "0.0.0.0:3000";
    info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind to address")?;
    
    axum::serve(listener, app)
        .await
        .context("Server error")?;

    Ok(())
}

/// Health check endpoint for k8s/docker
async fn health() -> impl IntoResponse {
    StatusCode::OK
}

/// Handle incoming GitHub webhooks
async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // Verify webhook signature
    if let Err(e) = verify_webhook(&state, &headers, &body) {
        tracing::warn!("Webhook verification failed: {:?}", e);
        return StatusCode::UNAUTHORIZED;
    }

    // Extract event type
    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // Parse webhook event
    let event = match WebhookEvent::try_from_header_and_body(event_type, &body) {
        Ok(e) => e,
        Err(e) => {
            tracing::error!("Failed to parse webhook event: {:?}", e);
            return StatusCode::BAD_REQUEST;
        }
    };

    // Process PR events asynchronously
    if let WebhookEventType::PullRequest = event.kind {
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = process_pull_request(state_clone, event).await {
                tracing::error!("Failed to process PR: {:?}", e);
            }
        });
        tracing::info!("Queued PR processing job");
    }

    StatusCode::ACCEPTED
}

/// Verify GitHub webhook HMAC signature
fn verify_webhook(state: &AppState, headers: &HeaderMap, body: &Bytes) -> Result<(), ()> {
    let sig = headers
        .get("X-Hub-Signature-256")
        .ok_or(())?
        .to_str()
        .map_err(|_| ())?;
    
    let remote = sig.strip_prefix("sha256=").ok_or(())?;

    let mut mac = HmacSha256::new_from_slice(state.webhook_secret.expose_secret().as_bytes())
        .map_err(|_| ())?;
    mac.update(body);
    let local = mac.finalize().into_bytes();

    let remote = hex::decode(remote).map_err(|_| ())?;

    if local.ct_eq(&remote).unwrap_u8() != 1 {
        return Err(());
    }

    Ok(())
}

/// Process pull request in background
async fn process_pull_request(state: Arc<AppState>, event: WebhookEvent) -> anyhow::Result<()> {
    let repo = event.repository.context("No repository in event")?;
    
    let pr_payload = match event.specific {
        WebhookEventPayload::PullRequest(p) => p,
        _ => return Ok(()), // Not a PR event
    };

    let installation_id = match event.installation.context("No installation in event")? {
        EventInstallation::Full(i) => i.id,
        EventInstallation::Minimal(i) => i.id,
    };
let repo_name = repo.name.clone();
let pr_number = pr_payload.pull_request.number;
println!("Processing PR #{} in repo: {}", pr_number, repo_name);

    // Create GitHub client
    let app_key = EncodingKey::from_rsa_pem(&state.private_key)
        .context("Failed to create RSA key from private key")?;
    
    let octo = Octocrab::builder()
        .app(state.app_id.into(), app_key)
        .build()
        .context("Failed to build GitHub client")?
        .installation(installation_id);

    let owner = repo.owner
        .context("No owner in repository")?
        .login;
    
    let repo_name = repo.name.clone();
    let pr_number = pr_payload.pull_request.number;

    // Get PR diff
    let diff = github::get_pr_diff(&octo, &owner, &repo_name, pr_number)
        .await
        .context("Failed to get PR diff")?;

    // Process the diff
    let result = github::process_diff(&diff)
        .await
        .context("Failed to process diff")?;

    // Store in database
    sqlx::query(
        r#"
        INSERT INTO audit_logs 
        (repo_name, pr_number, status, risk_score, report, created_at) 
        VALUES ($1, $2, $3, $4, $5, NOW())
        "#
    )
    .bind(&repo_name)
    .bind(pr_number as i32)
    .bind(&result.status)
    .bind(result.risk_score as i32)
    .bind(serde_json::to_value(&result)?)
    .execute(&state.db)
    .await
    .context("Failed to insert audit log")?;

    info!(
        "Audit log stored for PR #{}/{}",
        repo_name, pr_number
    );

    // Post review to GitHub
    github::post_review(&octo, &owner, &repo_name, pr_number, &result)
        .await
        .context("Failed to post review")?;

    info!(
        "Review posted for PR #{}/{}",
        repo_name, pr_number
    );

    Ok(())
}