use octocrab::Octocrab;
use anyhow::Context;
use serde_json::json;

pub async fn get_pr_context(client: &Octocrab, owner: &str, repo: &str, pr_number: u64) -> anyhow::Result<String> {
    let diff = client.pulls(owner, repo).get_diff(pr_number).await?;
    Ok(diff)
}

pub async fn run_privacy_audit(diff: &str) -> anyhow::Result<String> {
    let api_key = std::env::var("OPENAI_API_KEY").context("OPENAI_API_KEY missing")?;
    let client = reqwest::Client::new();

    let response = client.post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "gpt-4o",
            "messages": [
                {"role": "system", "content": "You are GhostHealth Guard. Scan for PHI leaks (names, SSNs, heart rates)."},
                {"role": "user", "content": diff}
            ]
        }))
        .send()
        .await?;

    let json_res: serde_json::Value = response.json().await?;
    Ok(json_res["choices"][0]["message"]["content"].as_str().unwrap_or("Error").to_string())
}

pub async fn post_review(
    client: &Octocrab, 
    owner: &str, 
    repo: &str, 
    pr_number: u64, 
    report: &str, 
    has_violations: bool
) -> anyhow::Result<()> {
    let action = if has_violations {
     let event = if has_violations {
        "REQUEST_CHANGES"
    } else {
        "COMMENT"
    };

    let route = format!("/repos/{owner}/{repo}/pulls/{pr_number}/reviews");
    let body = json!({
        "body": report,
        "event": event,
    });

    let _: serde_json::Value = client.post(route, Some(&body)).await?;
        
    Ok(())
}
