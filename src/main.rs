// src/main.rs
mod models;
mod github;

use axum::{body::Bytes, extract::State, http::{HeaderMap, StatusCode}, routing::post, Router, response::IntoResponse};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use secrecy::{SecretString, ExposeSecret};
use octocrab::models::webhook_events::{WebhookEvent, WebhookEventType};
use octocrab::Octocrab;

#[derive(Clone)]
struct AppState {
    webhook_secret: SecretString,
    app_id: u64,
    private_key: Vec<u8>, // Load your .pem file into this
}

type HmacSha256 = Hmac<Sha256>;

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    let app_id = std::env::var("GITHUB_APP_ID")
        .expect("GITHUB_APP_ID must be set")
        .parse::<u64>()
        .expect("APP_ID must be a number");
    
    let webhook_secret = std::env::var("GITHUB_WEBHOOK_SECRET")
        .expect("GITHUB_WEBHOOK_SECRET must be set");

    let state = AppState {
        webhook_secret: SecretString::new(webhook_secret.into()),
        app_id,
        private_key: std::fs::read("private-key.pem").expect("private-key.pem not found"),
    };
    
    // ... rest of your axum setup
}

    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .with_state(state);

    println!("GhostHealth Guard sentinel listening on 0.0.0.0:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // --- 1. SIGNATURE VERIFICATION ---
    let signature = headers.get("X-Hub-Signature-256").and_then(|h| h.to_str().ok()).unwrap_or_default();
    let remote_sha = signature.strip_prefix("sha256=").unwrap_or_default();
    let remote_sig_bytes = hex::decode(remote_sha).unwrap_or_default();

    let mut mac = HmacSha256::new_from_slice(state.webhook_secret.expose_secret().as_bytes()).unwrap();
    mac.update(&body);
    let expected_sig = mac.finalize().into_bytes();

    if expected_sig.ct_eq(&remote_sig_bytes.into()).unwrap_u8() != 1 {
        return (StatusCode::UNAUTHORIZED, "Invalid signature").into_response();
    }

    // --- 2. EVENT HANDLING ---
    let event_type = headers.get("X-GitHub-Event").and_then(|h| h.to_str().ok()).unwrap_or_default();
    
    if let Ok(event) = WebhookEvent::try_from_header_and_body(event_type, &body) {
        if let WebhookEventType::PullRequest = event.kind {
            // Get Installation ID to act as the App
            if let Some(installation) = event.installation {
                let octo = Octocrab::builder()
                    .app(state.app_id.into(), jsonwebtoken::EncodingKey::from_rsa_pem(&state.private_key).unwrap())
                    .build().unwrap();
                
                let installation_client = octo.installation(installation.id);

                // Example logic for "opened" PRs
               if let Some(repo) = event.repository {
    let owner = repo.owner.login;
    let name = repo.name;

    // Dynamically get the PR number from the event payload
    if let WebhookEvent { content, .. } = event {
        if let Some(pr_event) = content.as_pull_request() {
            let pr_number = pr_event.pull_request.number;

           match github::get_pr_context(&installation_client, &owner, &name, pr_number).await {
    Ok(context) => {
        println!("Auditing PR #{}...", pr_number);

        // --- STEP 3: RUN THE AI AUDIT ---
        match github::run_privacy_audit(&context).await {
            Ok(report) => {
                let final_message = format!("### 👻 GhostHealth Guard Privacy Report\n\n{}", report);
                
                // Post the actual AI findings back to GitHub
                let _ = github::post_comment(
                    &installation_client, 
                    &owner, 
                    &name, 
                    pr_number, 
                    &final_message
                ).await;
            },
            Err(e) => {
                println!("AI Audit Error: {:?}", e);
                let _ = github::post_comment(
                    &installation_client, &owner, &name, pr_number,
                    "⚠️ **GhostHealth Guard Error**: Failed to complete AI audit."
                ).await;
            }
        }
    },
    Err(e) => println!("Error fetching context: {:?}", e),
    
}
                    }
                }
            }
        }
    }

    StatusCode::OK.into_response()
}