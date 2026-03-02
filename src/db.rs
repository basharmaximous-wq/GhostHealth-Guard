use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use anyhow::Context;

pub async fn init_db(database_url: &str) -> anyhow::Result<SqlitePool> {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .context("Failed to connect to DB")
}
