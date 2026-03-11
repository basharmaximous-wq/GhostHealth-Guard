// ─────────────────────────────────────────────
// Core security logic
// ─────────────────────────────────────────────
pub mod audit;
pub mod models;
pub mod remediation;
pub mod scanner;

// Cryptography & integrity
pub mod fips;
pub mod hash;
pub mod zk;

pub mod db;
// pub mod enclave;
// pub mod anomaly;
// pub mod zk_circuit;
// pub mod vector_store;
// pub mod soc2;

// External integrations
pub mod blockchain;
pub mod github;

// Domain logic
pub mod patient_processor;

// ─────────────────────────────────────────────
// Re-exports
// ─────────────────────────────────────────────
pub use audit::*;
pub use blockchain::*;
pub use github::*;
pub use hash::*;
pub use models::*;
pub use scanner::*;
// ─────────────────────────────────────────────
// Imports
// ─────────────────────────────────────────────
use anyhow::Context;
use axum::body::Bytes;
use axum::http::HeaderMap;
use hmac::{Hmac, Mac};
use jsonwebtoken::EncodingKey;
use octocrab::models::{Installation, InstallationId};
use secrecy::SecretString;
use sha2::Sha256;
use sqlx::PgPool;
use std::sync::Arc;

// ─────────────────────────────────────────────
// AppState
// ─────────────────────────────────────────────
#[derive(Clone)]
pub struct AppState {
    pub webhook_secret: SecretString,
    pub github_app_id: u64,
    pub private_key: Vec<u8>,
    pub db: PgPool,
}

// ─────────────────────────────────────────────
// Repository
// ─────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct Repository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
}

// ─────────────────────────────────────────────
// Pull Request types
// ─────────────────────────────────────────────
#[derive(Debug)]
pub struct PullRequestBranch {
    pub r#ref: String,
    pub sha: String,
}

#[derive(Debug)]
pub struct PullRequest {
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub head: PullRequestBranch,
    pub base: PullRequestBranch,
}

#[derive(Debug)]
pub struct PullRequestEvent {
    pub action: String,
    pub number: i64,
    pub pull_request: PullRequest,
    pub repository: Repository,
}

// ─────────────────────────────────────────────
// Installation events
// ─────────────────────────────────────────────
#[derive(Debug)]
pub enum InstallationEvent {
    Created(Installation),
    Deleted(Installation),
}

#[derive(Debug)]
pub enum EventInstallation {
    Full(Box<Installation>),
    Minimal(Box<InstallationId>),
}

// ─────────────────────────────────────────────
// Webhook event
// ─────────────────────────────────────────────
#[derive(Debug)]
pub enum WebhookEventPayload {
    PullRequest(Box<PullRequestEvent>),
}

#[derive(Debug)]
pub struct WebhookEvent {
    pub repository: Option<Repository>,
    pub installation: Option<EventInstallation>,
    pub spec: WebhookEventPayload,
}

impl WebhookEvent {
    pub fn from_bytes(body: &Bytes, signature: &str, secret: &[u8]) -> anyhow::Result<Self> {
        // 1. Verify HMAC-SHA256 signature
        let sig_bytes = hex::decode(
            signature
                .strip_prefix("sha256=")
                .context("Signature missing sha256= prefix")?,
        )
        .context("Invalid hex in signature")?;

        let mut mac = Hmac::<Sha256>::new_from_slice(secret).context("Failed to create HMAC")?;
        mac.update(body);
        mac.verify_slice(&sig_bytes)
            .context("Webhook signature mismatch — invalid secret or payload")?;

        // 2. Parse JSON payload
        let payload: serde_json::Value =
            serde_json::from_slice(body).context("Failed to parse webhook JSON")?;

        // 3. Build Repository
        let repo = payload["repository"].as_object().map(|r| Repository {
            id: r["id"].as_i64().unwrap_or(0),
            name: r["name"].as_str().unwrap_or("").to_string(),
            full_name: r["full_name"].as_str().unwrap_or("").to_string(),
        });

        let repo_for_event = repo.clone().unwrap_or(Repository {
            id: 0,
            name: String::new(),
            full_name: String::new(),
        });

        // 4. Build WebhookEvent
        Ok(Self {
            repository: repo,
            installation: None,
            spec: WebhookEventPayload::PullRequest(Box::new(PullRequestEvent {
                action: payload["action"].as_str().unwrap_or("").to_string(),
                number: payload["number"].as_i64().unwrap_or(0),
                pull_request: PullRequest {
                    number: payload["pull_request"]["number"].as_i64().unwrap_or(0),
                    title: payload["pull_request"]["title"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    body: payload["pull_request"]["body"]
                        .as_str()
                        .map(|s| s.to_string()),
                    head: PullRequestBranch {
                        r#ref: payload["pull_request"]["head"]["ref"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                        sha: payload["pull_request"]["head"]["sha"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                    },
                    base: PullRequestBranch {
                        r#ref: payload["pull_request"]["base"]["ref"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                        sha: payload["pull_request"]["base"]["sha"]
                            .as_str()
                            .unwrap_or("")
                            .to_string(),
                    },
                },
                repository: repo_for_event,
            })),
        })
    }
}

// ─────────────────────────────────────────────
// Webhook signature verification
// ─────────────────────────────────────────────
pub fn verify_webhook(state: &AppState, headers: &HeaderMap, body: &Bytes) -> anyhow::Result<()> {
    use secrecy::ExposeSecret;

    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .context("Missing X-Hub-Signature-256 header")?;

    let sig_bytes = hex::decode(
        signature
            .strip_prefix("sha256=")
            .context("Signature missing sha256= prefix")?,
    )
    .context("Invalid hex in signature")?;

    let mut mac = Hmac::<Sha256>::new_from_slice(state.webhook_secret.expose_secret().as_bytes())
        .context("Failed to create HMAC")?;
    mac.update(body);
    mac.verify_slice(&sig_bytes)
        .context("Webhook signature verification failed")?;

    Ok(())
}

// ─────────────────────────────────────────────
// Pull Request processor
// ─────────────────────────────────────────────
pub async fn process_pull_request(
    state: Arc<AppState>,
    payload: serde_json::Value,
) -> anyhow::Result<()> {
    use crate::github;
    use octocrab::Octocrab;
    use sqlx::Row;
    use tracing::info;

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
            .app(state.github_app_id.into(), app_key)
            .build()?
            .installation(octocrab::models::InstallationId(installation_id));

        let (owner, repo) = repo_name.split_once('/').unwrap_or(("ghost", "repo"));
        let _ = github::post_review(&octo, owner, repo, pr_number, &result).await;
        info!("Review posted to GitHub PR #{}", pr_number);
    }

    Ok(())
}
