// src/github.rs
use octocrab::Octocrab;
use crate::models::PullRequestAuditContext;

pub async fn get_pr_context(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    pr_number: u64,
) -> anyhow::Result<PullRequestAuditContext> {
    // Fetch PR Metadata
    let pr = octocrab.pulls(owner, repo).get(pr_number).await?;

    // Fetch the raw Diff
    let diff = octocrab.pulls(owner, repo).get_diff(pr_number).await?;

    Ok(PullRequestAuditContext {
        title: pr.title.unwrap_or_default(),
        description: pr.body.unwrap_or_default(),
        diff,
        repo_owner: owner.to_string(),
        repo_name: repo.to_string(),
        pr_number,
    })
}

pub async fn post_comment(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    pr_number: u64,
    message: &str,
) -> anyhow::Result<()> {
    octocrab
        .issues(owner, repo)
        .create_comment(pr_number, message)
        .await?;
    Ok(())
}
pub async fn run_privacy_audit(context: &crate::models::PullRequestAuditContext) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let api_key = std::env::var("OPENAI_API_KEY").expect("AI API Key not set");

    // The "GhostHealth Guard" System Prompt
    let system_prompt = "You are GhostHealth Guard, a high-compliance HIPAA privacy auditor. \
        Review the following Rust code diff for privacy leaks. \
        1. Flag logging of variables like name, DOB, or SSN. \
        2. Ensure structs with sensitive data don't implement Debug/Display without masking. \
        3. Check for unencrypted data transit. \
        4. Focus on 'unsafe' blocks that handle patient buffers. \
        Be concise and professional.";

    let user_prompt = format!(
        "PR Title: {}\nPR Description: {}\n\nDiff Content:\n{}",
        context.title, context.description, context.diff
    );

    let response = client
        .post("https://api.openai.com/v1/chat/completions") // You can use Groq or local Ollama too
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "model": "gpt-4o", // Or your fine-tuned model
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "temperature": 0.1 // Keep it low for consistent, professional auditing
        }))
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;
    let report = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("Audit failed to generate a report.")
        .to_string();

    Ok(report)
}