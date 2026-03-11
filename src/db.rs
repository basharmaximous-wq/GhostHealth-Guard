use anyhow::Context;
use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn init_db(database_url: &str) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .context("Failed to connect to DB")
}
