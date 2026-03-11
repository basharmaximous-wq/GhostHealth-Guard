use anyhow::Context;
use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn init_db(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(database_url)
        .await
        .context("Failed to connect to PostgreSQL database")?;

    // Automatic Migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run database migrations")?;

    Ok(pool)
}
