use octocrab::{params::repos::Reference, Octocrab};
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

pub async fn open_remediation_pr(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    base_branch: &str,
    fix_content: String,
) -> anyhow::Result<()> {
    let branch_name = "ghosthealth-remediation";
    let target_path = "src/patient.rs";

    // Get the base reference
    let base_ref = client
        .repos(owner, repo)
        .get_ref(&Reference::Branch(base_branch.to_string()))
        .await?;

    let base_sha = match base_ref.object {
        octocrab::models::repos::Object::Commit { sha, .. }
        | octocrab::models::repos::Object::Tag { sha, .. } => sha,
        _ => return Err(anyhow::anyhow!("Unsupported reference object returned by GitHub")),
    };

    // Create branch if it doesn't exist
    if let Err(err) = client
        .repos(owner, repo)
        .create_ref(&Reference::Branch(branch_name.to_string()), &base_sha)
        .await
    {
        let err_string = err.to_string();
        if !err_string.contains("Reference already exists") {
            return Err(err.into());
        }
    }

    // Try to update existing file or create new one
    let existing_file = client
        .repos(owner, repo)
        .get_content()
        .path(target_path)
        .r#ref(branch_name)
        .send()
        .await;

    match existing_file {
        Ok(page) => {
            let file = page
                .items
                .first()
                .ok_or_else(|| anyhow::anyhow!("Expected existing file metadata for {}", target_path))?;

            client
                .repos(owner, repo)
                .update_file(
                    target_path,
                    "Auto-remediation: remove PHI logging",
                    &fix_content,
                    file.sha.clone(),
                )
                .branch(branch_name)
                .send()
                .await?;
        }
        Err(_) => {
            client
                .repos(owner, repo)
                .create_file(
                    target_path,
                    "Auto-remediation: remove PHI logging",
                    &fix_content,
                )
                .branch(branch_name)
                .send()
                .await?;
        }
    }

    // Create the pull request - CORRECT for octocrab 0.38
client
    .pulls(owner, repo)
    .create(
        "Compliance Fix: Remove PHI logging",  // Title FIRST
        branch_name,                            // Head branch (your feature branch)
        base_branch                             // Base branch (where to merge into)
    )
    .body("Automated remediation for PHI leak")
    .send()
    .await?;

    Ok(())
}