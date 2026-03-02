use crate::models::AuditResult;
use crate::{audit, scanner};
use octocrab::Octocrab;
use serde_json::json;

pub async fn get_pr_diff(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    pr_number: u64,
) -> anyhow::Result<String> {
    Ok(client.pulls(owner, repo).get_diff(pr_number).await?)
}

pub async fn process_diff(diff: &str) -> anyhow::Result<AuditResult> {
    // 1. Run deterministic regex scan
    let mut issues = scanner::deterministic_scan(diff);
    
    // 2. Run LLM review (Gemini 1.5 Flash Free Tier)
    let mut ai = audit::llm_review(diff).await?;
    
    // If AI was blocked, we still report what the regex found
    issues.append(&mut ai.issues);

    let risk_score = if ai.status == "VIOLATION" {
        ai.risk_score.max((issues.len() * 10).min(100) as u8)
    } else {
        (issues.len() * 15).min(100) as u8
    };

    let status = if risk_score > 30 || ai.status == "VIOLATION" {
        "VIOLATION"
    } else {
        "CLEAN"
    };

    Ok(AuditResult {
        status: status.into(),
        risk_score,
        issues,
    })
}

pub async fn post_review(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    pr: u64,
    result: &AuditResult,
) -> anyhow::Result<()> {
    // Determine the header based on status
    let header = match result.status.as_str() {
        "VIOLATION" => "⚠️ **GhostHealth Guard: Action Required**",
        "BLOCKED" => "ℹ️ **GhostHealth Guard: Scan Partial**",
        _ => "✅ **GhostHealth Guard: Clean**",
    };

    let body = json!({
        "body": format!(
            "{}\n\n**Status:** {}\n**Risk Score:** {}/100\n\n### Findings:\n{:#?}",
            header, result.status, result.risk_score, result.issues
        ),
        "event": if result.status == "VIOLATION" { "REQUEST_CHANGES" } else { "COMMENT" }
    });

    let route = format!("/repos/{owner}/{repo}/pulls/{pr}/reviews");
    
    // Post the review to GitHub
    client.post::<_, serde_json::Value>(route, Some(&body)).await?;
    
    Ok(())
}
pub fn post_review_dummy() {
    println!("Mocking GitHub Review: Analysis report would be posted here.");
}
