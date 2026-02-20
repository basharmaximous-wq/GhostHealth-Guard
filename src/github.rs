use octocrab::Octocrab;
use octocrab::models::pulls::ReviewAction; // Correct 0.38 location
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
        ReviewAction::RequestChanges
    } else {
        ReviewAction::Comment
    };
    
    // Use the reviews() builder in Octocrab 0.38
    client.pulls(owner, repo)
        .reviews()
        .create(pr_number)
        .body(report)
        .event(action)
        .send()
        .await?;
        
    Ok(())
}



