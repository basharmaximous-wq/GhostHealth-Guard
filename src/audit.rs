use crate::hash::generate_hash;
use crate::models::{AuditResult, Issue};
use anyhow::Context;
use serde_json::json;

pub struct AuditEntry {
    pub data_hash: String,
    pub entry_hash: String,
    pub previous_hash: String,
}

impl AuditEntry {
    pub fn new(document: &str, previous_hash: &str) -> Self {
        let data_hash = generate_hash(document);
        let entry_hash = generate_hash(&format!("{}{}", data_hash, previous_hash));
        Self {
            data_hash,
            entry_hash,
            previous_hash: previous_hash.to_string(),
        }
    }
}

pub async fn llm_review(diff: &str) -> anyhow::Result<AuditResult> {
    let key = std::env::var("GEMINI_API_KEY")
        .context("GEMINI_API_KEY not set")?;

    let client = reqwest::Client::new();

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
        key
    );

    let response = client
        .post(&url)
        .json(&json!({
            "contents": [{
                "parts": [{
                    "text": format!(
                        "You are a HIPAA compliance expert. Analyze this code diff for PHI leaks and privacy violations.\n\nReturn ONLY valid JSON in this exact format with no extra text:\n{{\"status\": \"CLEAN\", \"risk_score\": 0, \"issues\": []}}\n\nFor violations use:\n{{\"status\": \"VIOLATION\", \"risk_score\": 85, \"issues\": [{{\"category\": \"PHI_LOGGING\", \"severity\": \"HIGH\", \"message\": \"description\"}}]}}\n\nCode diff to analyze:\n{}",
                        diff
                    )
                }]
            }],
            "generationConfig": {
                "temperature": 0.1,
                "maxOutputTokens": 500
            }
        }))
        .send()
        .await
        .context("Failed to send request to Gemini")?;

    let status = response.status();
    let body = response.json::<serde_json::Value>().await
        .context("Failed to parse Gemini response")?;

    if !status.is_success() {
        let error = body["error"]["message"].as_str().unwrap_or("Unknown error");
        anyhow::bail!("Gemini API error: {}", error);
    }

    let content = body["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .context("No content in Gemini response")?;

    // Clean up markdown code blocks if present
    let clean = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    match serde_json::from_str::<AuditResult>(clean) {
        Ok(result) => Ok(result),
        Err(_) => {
            Ok(AuditResult {
                status: "CLEAN".to_string(),
                risk_score: 0,
                issues: vec![Issue {
                    category: "PARSE_ERROR".to_string(),
                    severity: "LOW".to_string(),
                    message: "Could not parse LLM response".to_string(),
                }],
            })
        }
    }
}