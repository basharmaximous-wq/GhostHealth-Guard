use octocrab::Octocrab;
use octocrab::models::repos::Reference;

pub async fn open_remediation_pr(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    base_branch: &str,
    fix_content: String,
) -> anyhow::Result<()> {

    let branch_name = "ghosthealth-remediation";

    // 1️⃣ Get base branch reference (to get latest commit SHA)
    let base_ref = client
        .repos(owner, repo)
        .get_ref(&format!("heads/{}", base_branch))
        .await?;

    let base_sha = base_ref.object.sha;

    // 2️⃣ Create new branch from base SHA
    client
        .repos(owner, repo)
        .create_ref(
            &Reference {
                ref_field: format!("refs/heads/{}", branch_name),
                node_id: None,
                url: None,
                object: None,
            }
        )
        .send()
        .await?;

    // 3️⃣ Get file to obtain current SHA
    let file = client
        .repos(owner, repo)
        .get_content()
        .path("src/patient.rs")
        .r#ref(branch_name)
        .send()
        .await?;

    let file_sha = file.items[0].sha.clone();

    // 4️⃣ Update file on new branch
    client
        .repos(owner, repo)
        .update_file(
            "src/patient.rs",
            "Auto-remediation: remove PHI logging",
        )
        .content(fix_content)
        .sha(file_sha)
        .branch(branch_name)
        .send()
        .await?;

    // 5️⃣ Open PR
    client
        .pulls(owner, repo)
        .create("Compliance Fix", branch_name, base_branch)
        .body("Automated remediation for PHI leak")
        .send()
        .await?;

    Ok(())
}