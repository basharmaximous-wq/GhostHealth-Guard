use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(FromRow)]
pub struct AuditRecord {
    pub id: Uuid,
    pub repo_name: String,
    pub pr_number: i32,
    pub status: String,
    pub report: String,
    pub created_at: DateTime<Utc>,
}
