pub mod config;
pub mod db;
pub mod error;
pub mod auth;
pub mod api_key;
pub mod modules;
pub mod orchestrator;
pub mod handlers;
pub mod middleware;

pub use modules::error_capture;
pub use modules::projects;
pub use modules::storage;
pub use modules::alert;

use actix_web::{web, App, HttpServer};
use anyhow::Result;
use config::Config;
use db::create_pool;
use tracing::error;

#[actix_web::main]
async fn main() -> Result<()> {
    // Init logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::load()?;
    // Try to create DB pool; allow startup to continue even if DB is unavailable
    let pool_result_raw = create_pool(&config).await;
    let pool_result: Option<_> = match pool_result_raw {
        Ok(p) => Some(p),
        Err(e) => {
            error!("Database unavailable at startup: {}", e);
            None
        }
    };

    let api_key_cache = crate::auth::cache::ApiKeyCache::new(std::time::Duration::from_secs(300));

    println!("🚀 FaultReport starting on http://0.0.0.0:8000");
    println!("📊 Health check: http://localhost:8000/api/healthz");

    HttpServer::new(move || {
        let app_config = config.clone();
        let app_cache = api_key_cache.clone();

        if let Some(pool) = &pool_result {
            let app_pool = pool.clone();
            App::new()
                .app_data(web::Data::new(app_pool))
                .app_data(web::Data::new(app_cache))
                .configure(orchestrator::config(&app_config))
        } else {
            // Start server without DB pool; routes requiring DB will fail at runtime.
            App::new()
                .app_data(web::Data::new(app_cache))
                .configure(orchestrator::config(&app_config))
        }
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await?;

    Ok(())
}
