use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: Uuid,
    pub repo_name: String,
    pub pr_number: i32,
    pub status: String, // e.g., "CLEAN", "VIOLATION"
    pub report: String,
    pub created_at: DateTime<Utc>,
}

// Pro Tip: Keep your database query logic near the model
impl AuditRecord {
    pub async fn create(
        pool: &sqlx::PgPool,
        repo_name: &str,
        pr_number: i32,
        status: &str,
        report: &str,
    ) -> anyhow::Result<Self> {
        let record = sqlx::query_as!(
            AuditRecord,
            r#"
            INSERT INTO audit_logs (repo_name, pr_number, status, report)
            VALUES ($1, $2, $3, $4)
            RETURNING id, repo_name, pr_number, status, report, created_at
            "#,
            repo_name,
            pr_number,
            status,
            report
        )
        .fetch_one(pool)
        .await?;

        Ok(record)
    }
}
