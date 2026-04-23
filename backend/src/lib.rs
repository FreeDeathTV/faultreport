pub mod config;
pub mod db;
pub mod error;
pub mod modules;
pub mod orchestrator;
pub mod handlers;
pub mod middleware;
pub mod auth;
pub mod api_key;

// Convenient re-exports for consumers (including integration tests)
pub use modules::{alert, error_capture, projects, storage};

// PgPool extensions for Redis access
use sqlx::PgPool;
use anyhow::Result as AnyhowResult;

// Provide a stub extension trait so code calling `get_redis_pool` compiles
// while Redis initialization is optional for local development.
//
// TODO: Replace this stub with a real typed Redis pool or an optional
// feature flag that enables Redis-backed behavior. Consumers should be
// able to opt into Redis via configuration or feature flags.
pub trait PgPoolExt {
    fn get_redis_pool(&self) -> AnyhowResult<&()> {
        Err(anyhow::anyhow!("Redis pool not initialized"))
    }
}

impl PgPoolExt for PgPool {}
