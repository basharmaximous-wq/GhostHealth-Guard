use anyhow::Context;
use jsonwebtoken::EncodingKey;
use octocrab::Octocrab;

pub struct GitHubClient {
    pub crab: Octocrab,
}

impl GitHubClient {
    pub async fn new() -> anyhow::Result<Self> {
        // 1. Load App ID from environment
        let app_id = octocrab::models::AppId(
            std::env::var("GITHUB_APP_ID")
                .context("GITHUB_APP_ID not set")?
                .parse::<u64>()
                .context("GITHUB_APP_ID must be a number")?,
        );

        // 2. Load private key from file
        let key_path = std::env::var("PRIVATE_KEY_PATH")
            .unwrap_or_else(|_| "app/ghosthealth-guard-dev.2026-02-21.private-key.pem".to_string());

        let pem = std::fs::read(&key_path)
            .with_context(|| format!("Failed to read private key from {}", key_path))?;

        let key = EncodingKey::from_rsa_pem(&pem)
            .context("Failed to parse RSA private key")?;

        // 3. Build octocrab client authenticated as GitHub App
        let crab = Octocrab::builder()
            .app(app_id, key)
            .build()
            .context("Failed to build GitHub client")?;

        Ok(Self { crab })
    }

    /// Post a review comment on a PR
    pub async fn post_comment(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        body: &str,
    ) -> anyhow::Result<()> {
        self.crab
            .issues(owner, repo)
            .create_comment(pr_number, body)
            .await
            .context("Failed to post PR comment")?;

        Ok(())
    }
}
