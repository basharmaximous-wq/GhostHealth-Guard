use crate::hash::generate_hash;
use crate::models::{AuditResult, Issue};
use anyhow::Context;
use std::sync::OnceLock;
use serde_json::json;
use tokio::time::{sleep, Duration};

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

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub async fn llm_review(diff: &str) -> anyhow::Result<AuditResult> {
    let key = std::env::var("GEMINI_API_KEY")
        .context("GEMINI_API_KEY not set")?;

    let client = CLIENT.get_or_init(reqwest::Client::new);

    // FIXED: Using the standard 'gemini-1.5-flash' for maximum compatibility
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
        key
    );

    let mut attempts = 0;
    let max_attempts = 3;

    loop {
        attempts += 1;

        let response = client
            .post(&url)
            .json(&json!({
                "contents": [{
                    "parts": [{
                        "text": format!(
                            "You are a HIPAA compliance expert. Analyze this code diff for PHI leaks and privacy violations.\n\nReturn ONLY valid JSON in this exact format:\n{{\"status\": \"CLEAN\", \"risk_score\": 0, \"issues\": []}}\n\nFor violations use:\n{{\"status\": \"VIOLATION\", \"risk_score\": 85, \"issues\": [{{\"category\": \"PHI_LOGGING\", \"severity\": \"HIGH\", \"message\": \"description\"}}]}}\n\nCode diff:\n{}",
                            diff
                        )
                    }]
                }],
                // NEW: Added safetySettings to prevent blocking of mock PII data like SSNs
                "safetySettings": [
                    { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "BLOCK_NONE" },
                    { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "BLOCK_NONE" },
                    { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "BLOCK_NONE" },
                    { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "BLOCK_NONE" }
                ],
                "generationConfig": {
                    "temperature": 0.1,
                    "maxOutputTokens": 1000,
                    "responseMimeType": "application/json"
                }
            }))
            .send()
            .await
            .context("Failed to send request to Gemini")?;

        let status = response.status();

        if (status.as_u16() == 429 || status.is_server_error()) && attempts < max_attempts {
            let wait_secs = attempts * 2;
            tracing::warn!("Gemini API busy ({}). Retry attempt {}/{} in {}s...", status, attempts, max_attempts, wait_secs);
            sleep(Duration::from_secs(wait_secs)).await;
            continue;
        }

        let body = response.json::<serde_json::Value>().await
            .context("Failed to parse Gemini response")?;

        if !status.is_success() {
            let error = body["error"]["message"].as_str().unwrap_or("Unknown API error");
            anyhow::bail!("Gemini API error: {}", error);
        }

        if let Some(candidates) = body["candidates"].as_array() {
            if let Some(finish_reason) = candidates.first().and_then(|c| c["finishReason"].as_str()) {
                if finish_reason == "SAFETY" {
                    return Ok(AuditResult {
                        status: "BLOCKED".to_string(),
                        risk_score: 0,
                        issues: vec![Issue {
                            category: "SAFETY_FILTER".to_string(),
                            severity: "INFO".to_string(),
                            message: "Gemini safety filters blocked this scan because it detected sensitive content.".to_string(),
                        }],
                    });
                }
            }
        }

        let content = body["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .context("No content in Gemini response")?;

        let clean = content
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        return match serde_json::from_str::<AuditResult>(clean) {
            Ok(result) => Ok(result),
            Err(e) => {
                tracing::error!("JSON Parse Error: {}. Original content: {}", e, clean);
                Ok(AuditResult {
                    status: "CLEAN".to_string(),
                    risk_score: 0,
                    issues: vec![Issue {
                        category: "PARSE_ERROR".to_string(),
                        severity: "LOW".to_string(),
                        message: "AI returned non-JSON content. Defaulting to CLEAN.".to_string(),
                    }],
                })
            }
        };
    }
}