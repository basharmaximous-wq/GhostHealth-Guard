use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditResult {
    pub status: String,
    pub risk_score: u8,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Issue {
    pub category: String,
    pub severity: String,
    pub message: String,
}
