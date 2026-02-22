use axum::{extract::Json, http::{HeaderMap, StatusCode}};
use anyhow::{Context, Result};
use serde_json::Value;
use octocrab::models::webhooks::PullRequestWebhook;

// TEMPORARY MOCK - replace with your real audit::llm_review later
#[derive(Debug)]
pub struct Review {
    pub status: String,
    pub risk_score: u8,
}

pub async fn llm_review_mock(diff: &str) -> Result<Review> {
    println!("Mock LLM reviewing: {}", diff.lines().next().unwrap_or(""));
    Ok(Review {
        status: "CLEAN".to_string(),  // Or "VIOLATION"
        risk_score: 2,  // 0-10
    })
}

pub async fn handle_webhook(
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<String, StatusCode> {
    println!("Webhook received!");

    // Log event type (your existing code)
    if let Some(event) = headers.get("X-GitHub-Event") {
        println!("Event: {:?}", event.to_str().unwrap_or("unknown"));
    }

    // Only process pull_request events (opened/synchronize)
    let event = headers.get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    
    if event != "pull_request" {
        println!("Ignoring non-PR event: {}", event);
        return Ok("OK - ignored".to_string());
    }

    println!("Processing PR webhook: {:?}", payload);

    // Parse GitHub PR payload
    let pr_payload: PullRequestWebhook = serde_json::from_value(payload.clone())
        .context("Invalid PR webhook payload")?;

    let pr_number = pr_payload.pull_request.number as i64;
    let repo_name = pr_payload.repository.full_name
        .ok_or(StatusCode::BAD_REQUEST)?;

    println!("Auditing PR #{} in {}", pr_number, repo_name);

    // Create your GitHubClient
    let gh = crate::github_client::GitHubClient::new().await?;

    // Build PR diff from title + body
    let diff = format!(
        "PR #{} — {}\n{}",
        pr_number,
        pr_payload.pull_request.title,
        pr_payload.pull_request.body.unwrap_or_default()
    );

    // Run privacy review (mock for now)
    let review = llm_review_mock(&diff).await?;

    // Post audit comment to PR
    let (owner, repo_slug) = repo_name
        .split_once('/')
        .context("Invalid repo format: owner/repo")?;

    let comment = format!(
        r#"🔒 **GhostHealth Guard — Privacy Audit**

**Status:** `{}`  
**Risk Score:** `{}/10`  
**Reviewed:** {}

👻 All clear! 👻"#,
        review.status,
        review.risk_score,
        chrono::Local::now().format("%Y-%m-%d %H:%M")
    );

    gh.post_comment(owner, repo_slug, pr_number as u64, &comment)
        .await
        .context("Failed to post PR comment")?;

    println!("✅ Posted audit comment to PR #{} in {}", pr_number, repo_name);
    Ok(format!("Audit posted to PR #{}", pr_number))
}
