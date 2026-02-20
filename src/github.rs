use octocrab::Octocrab;
use serde_json::json;
use crate::{scanner, audit};
use crate::models::{AuditResult};

pub async fn get_pr_diff(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    pr_number: u64,
) -> anyhow::Result<String> {
    Ok(client.pulls(owner, repo).get_diff(pr_number).await?)
}

pub async fn process_diff(diff: &str) -> anyhow::Result<AuditResult> {
    let mut issues = scanner::deterministic_scan(diff);
    issues.extend(scanner::run_semgrep());

    let mut ai = audit::llm_review(diff).await?;
    issues.append(&mut ai.issues);

    let risk_score = (issues.len() * 20).min(100) as u8;
    let status = if risk_score > 30 { "VIOLATION" } else { "CLEAN" };

    Ok(AuditResult { status: status.into(), risk_score, issues })
}

pub async fn post_review(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    pr: u64,
    result: &AuditResult,
) -> anyhow::Result<()> {

    let body = json!({
        "body": format!(
            "## GhostHealth Guard\nStatus: {}\nRisk Score: {}\nIssues:\n{:?}",
            result.status, result.risk_score, result.issues
        ),
        "event": if result.status == "VIOLATION" { "REQUEST_CHANGES" } else { "COMMENT" }
    });

    let route = format!("/repos/{owner}/{repo}/pulls/{pr}/reviews");
    client.post(route, Some(&body)).await?;

    Ok(())
}
