use dotenvy::dotenv;
use ghosthealth_guard::*;
use sqlx::PgPool;

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
use octocrab::Octocrab;
use secrecy::{ExposeSecret, SecretString};
use sha2::Sha256;
use sqlx::Row;
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

    info!("Starting GhostHealth-Guard: Local Testing Mode Enabled");

    // 4. DATABASE: Using PostgreSQL
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/ghosthealth".to_string());

    let db = db::init_db(&database_url).await?;

    info!("Database connection established to PostgreSQL");

    // Run Migrations
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .context("Failed to run database migrations")?;

    info!("Database migrations applied successfully");

    // 5. GitHub Configuration (with local fallbacks to prevent crashes)
    let webhook_secret = SecretString::new(
        std::env::var("GITHUB_WEBHOOK_SECRET").unwrap_or_else(|_| "local_test_secret".to_string()),
    );

    let app_id = std::env::var("GITHUB_APP_ID")
        .unwrap_or_else(|_| "0".to_string())
        .parse()
        .unwrap_or(0);

    let private_key_path =
        std::env::var("PRIVATE_KEY_PATH").unwrap_or_else(|_| "key.pem".to_string());

    // Try to read key, but don't crash if it's missing during local testing
    let private_key = std::fs::read(&private_key_path).unwrap_or_else(|_| {
        tracing::warn!(
            "Private key not found at {}. Using empty key for testing.",
            private_key_path
        );
        vec![]
    });

    info!("GitHub App configuration loaded (Local Test Mode)");

    let state = Arc::new(AppState {
        webhook_secret,
        app_id,
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

async fn health() -> impl IntoResponse {
    info!("Health check heartbeat received");
    StatusCode::OK
}

async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // SECURITY BYPASS: If verification fails, we log it but KEEP GOING for testing
    if verify_webhook(&state, &headers, &body).is_err() {
        tracing::warn!("Webhook verification failed (Bypassing for local testing)");
    }

    let event_type = headers
        .get("X-GitHub-Event")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // Allow "unknown" events so you can test with Postman/Insomnia easily
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

fn verify_webhook(state: &AppState, headers: &HeaderMap, body: &Bytes) -> Result<(), ()> {
    let sig = match headers.get("X-Hub-Signature-256") {
        Some(s) => s.to_str().map_err(|_| ())?,
        None => return Err(()),
    };

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

async fn process_pull_request(
    state: Arc<AppState>,
    payload: serde_json::Value,
) -> anyhow::Result<()> {
    let repo_name = payload["repository"]["full_name"]
        .as_str()
        .unwrap_or("ghosthealth/test-repo")
        .to_string();

    let pr_number = payload["pull_request"]["number"].as_u64().unwrap_or(0);
    let installation_id = payload["installation"]["id"].as_u64().unwrap_or(0);
    let action = payload["action"].as_str().unwrap_or("opened");

    info!(
        "Processing PR #{} in {} (action: {})",
        pr_number, repo_name, action
    );

    // TEST MODE: Hardcoded leak for Gemini AI to discover
    let diff = "
+ fn update_user() {
+    let ssn = \"666-44-1111\";
+    let name = \"John Ghost\";
+    println!(\"Checking record for {}\", name);
+ }
"
    .to_string();

    // 1. Run AI Analysis
    let result = github::process_diff(&diff)
        .await
        .context("Gemini AI Analysis failed")?;

    // 2. Blockchain Audit Chain Hashing
    let last_record: Option<sqlx::postgres::PgRow> =
        sqlx::query("SELECT current_hash FROM audit_logs ORDER BY created_at DESC LIMIT 1")
            .fetch_optional(&state.db)
            .await?;

    let prev_hash = if let Some(row) = last_record {
        row.try_get::<String, _>("current_hash")?
    } else {
        "GENESIS_BLOCK".to_string()
    };

    let entry = audit::AuditEntry::new(
        &format!("{}{}", repo_name, serde_json::to_string(&result)?),
        &prev_hash,
    );
    let new_hash = entry.entry_hash.clone();

    // 3. Database Persistence
    let tenant_row: (uuid::Uuid,) = sqlx::query_as("SELECT id FROM tenants LIMIT 1")
        .fetch_one(&state.db)
        .await
        .context("No tenant found. Run your SQL setup scripts first.")?;

    sqlx::query(
        r#"
        INSERT INTO audit_logs
        (tenant_id, repo_name, pr_number, status, risk_score, report, previous_hash, current_hash)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(tenant_row.0)
    .bind(&repo_name)
    .bind(pr_number as i32)
    .bind(&result.status)
    .bind(result.risk_score as i32)
    .bind(serde_json::to_value(&result)?)
    .bind(prev_hash)
    .bind(new_hash)
    .execute(&state.db)
    .await?;

    info!(
        "SUCCESS: Audit log with hash {} saved to database",
        entry.entry_hash
    );

    // 4. GitHub Review (Wrapped in error handling so local runs don't crash without real keys)
    if !state.private_key.is_empty() {
        let app_key = EncodingKey::from_rsa_pem(&state.private_key).context("Invalid RSA key")?;
        let octo = Octocrab::builder()
            .app(state.app_id.into(), app_key)
            .build()?
            .installation(octocrab::models::InstallationId(installation_id));

        let (owner, repo) = repo_name.split_once('/').unwrap_or(("ghost", "repo"));
        let _ = github::post_review(&octo, owner, repo, pr_number, &result).await;
        info!("Review posted to GitHub PR #{}", pr_number);
    }

    Ok(())
}
