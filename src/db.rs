use anyhow::Context;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn init_db(database_url: &str) -> anyhow::Result<SqlitePool> {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .context("Failed to connect to DB")
}
