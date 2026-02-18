// src/models.rs

#[derive(Debug)]
pub struct PullRequestAuditContext {
    pub title: String,
    pub description: String,
    pub diff: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub pr_number: u64,
}