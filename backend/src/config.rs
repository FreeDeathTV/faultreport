use anyhow::{Context, Result};
use dotenv::dotenv;
use std::env;
use std::collections::HashSet;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: Option<String>,
    pub allowed_origins: HashSet<String>,
    pub rust_log: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .context("DATABASE_URL must be set")?;

        let redis_url = env::var("REDIS_URL").ok();

        let allowed_origins_str = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "localhost:3000,localhost:5173,localhost:8000".to_string());
        let allowed_origins: HashSet<String> = allowed_origins_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            database_url,
            redis_url,
            allowed_origins,
            rust_log,
        })
    }
}

