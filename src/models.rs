use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: Uuid,
    pub repo_name: String,
    pub pr_number: i32,
    pub status: String,
    pub report: String,
    pub created_at: DateTime<Utc>,
}

// Struct for testing PHI leaks
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Patient {
    pub name: String,
    pub heart_rate: i32,
    pub ssn: String,
}
