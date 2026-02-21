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
// Re-exports (explicit is safer than wildcard)
// ─────────────────────────────────────────────
pub use audit::*;
pub use blockchain::*;
pub use github::*;
pub use hash::*;
pub use models::*;
pub use scanner::*;

// ─────────────────────────────────────────────
// Imports required by the functions below
// ─────────────────────────────────────────────
use anyhow::Context;
use axum::http::HeaderMap;
use axum::body::Bytes;
use jsonwebtoken::EncodingKey;
use octocrab::models::{Installation, InstallationId}; 
use std::sync::Arc;

// ─────────────────────────────────────────────
// AppState — shared server state
// ─────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct AppState {
    pub private_key: Vec<u8>,
    pub github_app_id: String,
}

// ─────────────────────────────────────────────
// Repository — minimal representation
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

/// Represents the installation field on an incoming webhook event.
/// GitHub sends either the full object or just the ID depending on event type.
#[derive(Debug)]
pub enum EventInstallation {
    Full(Box<Installation>),
    Minimal(Box<InstallationId>),
}

// ─────────────────────────────────────────────
// Webhook event — the single canonical type
// (previously you had this defined TWICE, once
//  as an enum and once as a struct — removed the
//  enum; the struct is what your code actually uses)
// ─────────────────────────────────────────────
#[derive(Debug)]
pub enum WebhookEventPayload {
    PullRequest(Box<PullRequestEvent>),
    // extend with other variants as needed
}

#[derive(Debug)]
pub struct WebhookEvent {
    pub repository: Option<Repository>,
    pub installation: Option<EventInstallation>,
    pub spec: WebhookEventPayload,
}

impl WebhookEvent {
    /// Parse and verify an incoming GitHub webhook payload.
    pub fn from_bytes(_body: &Bytes, _signature: &str, _secret: &[u8]) -> anyhow::Result<Self> {
        // TODO: HMAC-SHA256 verify signature, then serde_json::from_slice
        todo!("Implement webhook verification")
    }
}

// ─────────────────────────────────────────────
// Webhook signature verification
// ─────────────────────────────────────────────
pub fn verify_webhook(
    _state: &AppState,
    headers: &HeaderMap,
    _body: &Bytes,
) -> anyhow::Result<()> {
    let _signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .context("Missing X-Hub-Signature-256 header")?;

    // TODO: compute HMAC-SHA256(_body, app_secret) and compare to _signature
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
        // nothing to do for non-PR events
    };

    let _installation_id = match event.installation.context("No installation")? {
        EventInstallation::Full(i) => i.id,
        // InstallationId is a newtype — inner value is .0, not .id
        EventInstallation::Minimal(i) => *i,  
    };  

    let _repo_name = repo.name.clone();
    let _pr_number = pr_payload.pull_request.number;

    // Build the GitHub App JWT signing key
    let _app_key = EncodingKey::from_rsa_pem(&state.private_key)
        .context("Failed to build RSA encoding key — is PRIVATE_KEY_PATH correct?")?;

    // TODO: create octocrab client with JWT, post review comments on the PR

    Ok(())
}