// src/main.rs
// GhostHealth Guard - Privacy Auditor for Health Code PRs
// Handles GitHub App webhooks for Pull Request events

mod models;
mod github;

use axum::{
    body::Bytes,
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
use sqlx::postgres::PgPool;
use std::sync::Arc;
use sqlx::PgPool;

#[derive(Clone)]
struct AppState {
    webhook_secret: SecretString,
    app_id: u64,
    private_key: Vec<u8>,
    db: PgPool,
}

type HmacSha256 = Hmac<Sha256>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let pool = PgPool::connect(&std::env::var("DATABASE_URL")?)
        .await?;
    
    // Pass pool to axum::Router::with_state(AppState { pool })
    // Run migrations: sqlx::migrate!("./migrations").run(&pool).await?;
    
    Ok(())
}

    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL must be set")?;
    let pool = PgPool::connect(&database_url).await
        .context("Failed to connect to DB")?;

    let app_id: u64 = std::env::var("GITHUB_APP_ID")
        .context("GITHUB_APP_ID must be set")?
        .parse()
        .context("GITHUB_APP_ID must be a number")?;

    let webhook_secret = std::env::var("GITHUB_WEBHOOK_SECRET")
        .context("GITHUB_WEBHOOK_SECRET must be set")?;

    let private_key = std::fs::read("private-key.pem")
        .context("private-key.pem not found")?;

    let state = AppState {
        webhook_secret: SecretString::new(webhook_secret),
        app_id,
        private_key,
        db: pool,
    };

    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .with_state(Arc::new(state));

    println!("GhostHealth Guard listening on 0.0.0.0:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // 1. Verify webhook signature
    let signature = match headers.get("X-Hub-Signature-256")
        .and_then(|h| h.to_str().ok())
    {
        Some(sig) => sig,
        None => return (StatusCode::UNAUTHORIZED, "Missing signature").into_response(),
    };

    let remote_sha = match signature.strip_prefix("sha256=") {
        Some(sha) => sha,
        None => return (StatusCode::UNAUTHORIZED, "Invalid signature format").into_response(),
    };

    let remote_sig_bytes = match hex::decode(remote_sha) {
        Ok(bytes) => bytes,
        Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid signature hex").into_response(),
    };

    let mut mac = match HmacSha256::new_from_slice(state.webhook_secret.expose_secret().as_bytes()) {
        Ok(m) => m,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "HMAC init failed").into_response(),
    };

    mac.update(&body);
    let expected_sig = mac.finalize().into_bytes();

    if expected_sig.ct_eq(&remote_sig_bytes[..]).unwrap_u8() != 1 {
        return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response(); [web:23][web:27]
    }

    // 2. Parse event
    let event_type = headers.get("X-GitHub-Event")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    let event = match WebhookEvent::try_from_header_and_body(event_type, &body) {
        Ok(ev) => ev,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    }; [web:21]

    // 3. Handle PullRequest events only
    if let WebhookEventType::PullRequest = event.kind {
        if let Some(installation) = event.installation {
            // Create authenticated client for installation [web:26]
            let app_key = match jsonwebtoken::EncodingKey::from_rsa_pem(&state.private_key) {
                Ok(key) => key,
                Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Invalid private key").into_response(),
            };

            let octo = match Octocrab::builder()
                .app(state.app_id.into(), app_key)
                .build()
            {
                Ok(o) => o,
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            let installation_client = match octo.installation(installation.id) {
                Ok(client) => client,
                Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            };

            // Extract repo and PR details
            let repo = match event.repository {
                Some(r) => r,
                None => return StatusCode::BAD_REQUEST.into_response(),
            };

            let owner = repo.owner.login;
            let name = repo.name;

            let pr_event = match event.content.as_pull_request() {
                Some(pr) => pr,
                None => return StatusCode::BAD_REQUEST.into_response(),
            };

            let pr_number = pr_event.pull_request.number;

            // 4. Get PR context and run audit
            match github::get_pr_context(&installation_client, &owner, &name, pr_number).await {
                Ok(context) => {
                    println!("Auditing PR #{} in {}/{}", pr_number, owner, name);
                    match github::run_privacy_audit(&context).await {
                        Ok(report) => {
                            let final_message = format!("### 👻 GhostHealth Guard Privacy Report\n\n{}", report);
                            let has_violations = report.to_uppercase().contains("VIOLATION")
                                || report.to_uppercase().contains("CRITICAL")
                                || report.to_uppercase().contains("LEAK");

                            // Post review
                            if let Err(e) = github::post_review(
                                &installation_client,
                                &owner,
                                &name,
                                pr_number,
                                &format!("### 👻 GhostHealth Guard Audit\n\n{}", report),
                                has_violations,
                            ).await {
                                println!("Failed to post review: {:?}", e);
                            }

                            // Log to DB [web:28]
                            let status = if has_violations { "VIOLATION" } else { "CLEAN" };
                            if let Err(e) = sqlx::query!(
                                "INSERT INTO audit_logs (repo_name, pr_number, status, report) VALUES ($1, $2, $3, $4)",
                                name,
                                pr_number as i32,
                                status,
                                report
                            )
                            .execute(&state.db)
                            .await
                            {
                                println!("DB log error: {:?}", e);
                            }

                            // Post comment
                            let _ = github::post_comment(
                                &installation_client,
                                &owner,
                                &name,
                                pr_number,
                                &final_message,
                            ).await;
                        }
                        Err(e) => {
                            println!("AI Audit Error: {:?}", e);
                            let _ = github::post_comment(
                                &installation_client,
                                &owner,
                                &name,
                                pr_number,
                                "⚠️ **GhostHealth Guard Error**: Failed to complete privacy audit.",
                            ).await;
                        }
                    }
                }
                Err(e) => {
                    println!("Error fetching PR context: {:?}", e);
                }
            }
        }
    }

    StatusCode::OK.into_response()
}
