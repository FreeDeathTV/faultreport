use anyhow::{Result, Context};
use sqlx::{PgPool, postgres::PgPoolOptions, migrate};
use tracing::info;
use crate::config::Config;

pub async fn create_pool(config: &Config) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    // Redis integration optional — disabled by default in local/dev builds.
    // If `REDIS_URL` is provided in `Config`, Redis pool initialization
    // will be implemented later. For now skip Redis to allow startup.

    // Run migrations on startup
    info!("Running database migrations...");
    migrate!("./migrations")
        .run(&pool)
        .await
        .context("Failed to run migrations")?;

    info!("Database ready");
    Ok(pool)
}

