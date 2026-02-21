use crate::models::AuditResult;
use anyhow::Context;
use serde_json::json;

pub async fn llm_review(diff: &str) -> anyhow::Result<AuditResult> {
    let key = std::env::var("OPENAI_API_KEY")?;
    let client = reqwest::Client::new();

    let response = client.post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", key))
        .json(&json!({
            "model": "gpt-4o",
            "response_format": { "type": "json_object" },
            "messages": [
                {
                    "role": "system",
                    "content": "Return strict JSON: {status: CLEAN|VIOLATION, risk_score: 0-100, issues:[{category,severity,message}]}"
                },
                { "role": "user", "content": diff }
            ]
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let content = response["choices"][0]["message"]["content"]
        .as_str()
        .context("No content returned")?;

    Ok(serde_json::from_str(content)?)
}
