use sqlx::PgPool;
use std::fs;
use std::path::Path;
use sqlx::migrate::Migrator;
use std::sync::Arc;

pub async fn apply_migrations_from_dir(pool: &PgPool, migrations_dir: &str) -> Result<(), sqlx::Error> {
    let dir = Path::new(migrations_dir);
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|s| s == "sql").unwrap_or(false))
        .collect();

    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        let sql = fs::read_to_string(&path).map_err(|e| sqlx::Error::Protocol(format!("failed to read {}: {}", path.display(), e)))?;
        // Execute the SQL contents as one statement blob. Migrations with multiple statements should work too.
        sqlx::query(&sql).execute(pool).await?;
    }

    Ok(())
}

pub async fn setup_test_db() -> Result<PgPool, sqlx::Error> {
    let db_url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set for integration tests");
    let pool = PgPool::connect(&db_url).await?;
    // Prefer sqlx Migrator if available; fall back to applying SQL files directly.
    let migrator_path = Path::new("backend/migrations");
    let migrator_result = Migrator::new(migrator_path).await;
    match migrator_result {
        Ok(migrator) => {
            migrator.run(&pool).await?;
        }
        Err(_) => {
            // fallback: apply plain SQL files in order
            apply_migrations_from_dir(&pool, "backend/migrations").await?;
        }
    }
    Ok(pool)
}
