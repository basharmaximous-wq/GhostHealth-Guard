// ─────────────────────────────────────────────
// Core security logic
// ─────────────────────────────────────────────
pub mod scanner;
pub mod audit;
pub mod models;
pub mod remediation;

// Cryptography & integrity
pub mod hash;
pub mod zk;
pub mod fips;

// External integrations
pub mod github;
pub mod blockchain;

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
use axum::http::HeaderMap;
use axum::body::Bytes;
use hex;
use hmac::{Hmac, Mac};
use jsonwebtoken::EncodingKey;
use octocrab::models::{Installation, InstallationId};
use sha2::Sha256;
use std::sync::Arc;

// ─────────────────────────────────────────────
// AppState
// ─────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct AppState {
    pub private_key: Vec<u8>,
    pub github_app_id: String,
    pub webhook_secret: Vec<u8>,
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

        let mut mac = Hmac::<Sha256>::new_from_slice(secret)
            .context("Failed to create HMAC")?;
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
pub fn verify_webhook(
    state: &AppState,
    headers: &HeaderMap,
    body: &Bytes,
) -> anyhow::Result<()> {
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

    let mut mac = Hmac::<Sha256>::new_from_slice(&state.webhook_secret)
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
    event: WebhookEvent,
) -> anyhow::Result<()> {
    let repo = event.repository.context("No repository in event")?;

    let pr_payload = match event.spec {
        WebhookEventPayload::PullRequest(p) => p,
    };

    let repo_name = repo.name.clone();
    let pr_number = pr_payload.pull_request.number;
    println!("Processing PR #{} in repo: {}", pr_number, repo_name);

    let _entry = AuditEntry::new(&repo_name, "genesis");

    let _app_key = EncodingKey::from_rsa_pem(&state.private_key)
        .context("Failed to build RSA encoding key — is PRIVATE_KEY_PATH correct?")?;

    // TODO: create octocrab client with JWT, post review comments on the PR

    Ok(())
}