use octocrab::Octocrab;
use anyhow::Context;

pub async fn get_pr_context(client: &Octocrab, owner: &str, repo: &str, pr_number: u64) -> anyhow::Result<String> {
    let diff = client.pulls(owner, repo).get_diff(pr_number).await?;
    Ok(diff)
}

pub async fn run_privacy_audit(diff: &str) -> anyhow::Result<String> {
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let client = reqwest::Client::new();

    let system_prompt = "You are GhostHealth Guard. Scan the following code diff for PHI (Names, SSNs, Heart Rate, etc). 
    If you find a leak, start your response with 'VIOLATION'. If clean, say 'CLEAN'.";

    let response = client.post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "gpt-4",
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": diff}
            ]
        }))
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;
    Ok(json["choices"][0]["message"]["content"].as_str().unwrap_or("Audit Error").to_string())
}

pub async fn post_review(client: &Octocrab, owner: &str, repo: &str, pr_number: u64, report: &str, has_violations: bool) -> anyhow::Result<()> {
    let action = if has_violations { "REQUEST_CHANGES" } else { "COMMENT" };
    
    client.pulls(owner, repo)
        .create_review(pr_number)
        .body(report)
        .event(octocrab::params::pulls::ReviewAction::from(action))
        .send()
        .await?;
    Ok(())
}
