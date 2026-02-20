use octocrab::Octocrab;
use anyhow::Context;
use serde_json::json;

/// Fetches the raw diff of a Pull Request to scan for leaks.
pub async fn get_pr_context(client: &Octocrab, owner: &str, repo: &str, pr_number: u64) -> anyhow::Result<String> {
    // In octocrab 0.38, get_diff returns the raw string of the code changes
    let diff = client.pulls(owner, repo).get_diff(pr_number).await?;
    Ok(diff)
}

/// Sends the code diff to the AI for a privacy audit.
pub async fn run_privacy_audit(diff: &str) -> anyhow::Result<String> {
    let api_key = std::env::var("OPENAI_API_KEY").context("OPENAI_API_KEY not set in .env")?;
    let client = reqwest::Client::new();

    let system_prompt = "You are GhostHealth Guard, a HIPAA-compliant security auditor. 
    Scan the following code diff for PHI (Patient Names, SSNs, Heart Rates, Medical IDs). 
    If you find a leak, start your response with 'VIOLATION' and list the concerns. 
    If it is safe, simply respond with 'CLEAN'.";

    let response = client.post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "gpt-4o", // Using the latest 4o model for better accuracy
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": diff}
            ],
            "temperature": 0.0 // Keep it strict and consistent
        }))
        .send()
        .await?;

    let json_res: serde_json::Value = response.json().await?;
    let content = json_res["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("Audit Error: AI failed to respond")
        .to_string();
        
    Ok(content)
}

/// Posts the audit result back to the GitHub PR as a formal review.
pub async fn post_review(
    client: &Octocrab, 
    owner: &str, 
    repo: &str, 
    pr_number: u64, 
    report: &str, 
    has_violations: bool
) -> anyhow::Result<()> {
    // We explicitly use the ReviewAction enum from the pulls params
    use octocrab::params::pulls::ReviewAction;

    let action = if has_violations {
        ReviewAction::RequestChanges
    } else {
        ReviewAction::Comment
    };
    
    client.pulls(owner, repo)
        .create_review(pr_number)
        .body(report)
        .event(action)
        .send()
        .await?;
        
    Ok(())
}
