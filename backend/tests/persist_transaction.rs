use anyhow::Result;
use faultreport::modules::error_capture::{RawError, ErrorContext};
use faultreport::modules::storage;
use anyhow::Result;
use faultreport::modules::error_capture::{RawError, ErrorContext};
use faultreport::modules::storage;
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use uuid::Uuid;

#[tokio::test]
async fn test_persist_transactional() -> Result<()> {
    // Load DB URL from TEST_DATABASE_URL or DATABASE_URL
    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_default();

    if database_url.is_empty() {
        eprintln!("Skipping test: TEST_DATABASE_URL or DATABASE_URL not set");
        return Ok(());
    }

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Create a temp user and project
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, firebase_uid, email) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind("test-uid")
        .bind("test@example.com")
        .execute(&pool)
        .await?;

    let project_id = Uuid::new_v4();
    sqlx::query("INSERT INTO projects (id, created_by_user_id, name, api_key_hash, api_key_salt) VALUES ($1, $2, $3, $4, $5)")
        .bind(project_id)
        .bind(user_id)
        .bind("test-project")
        .bind("hash")
        .bind("salt")
        .execute(&pool)
        .await?;

    let raw = RawError {
        message: "Transactional test error".to_string(),
        stack: Some("at test:1".to_string()),
        context: ErrorContext {
            url: "https://example.com/test".to_string(),
            browser: None,
            os: None,
            user_id: None,
            custom: serde_json::json!({}),
        },
    };

    let normalized = faultreport::modules::error_capture::normalize(&raw)?;

    // First insert
    let (id1, was_dup1, count1) = storage::persist(&pool, project_id, normalized.clone()).await?;
    assert_eq!(was_dup1, false);
    assert_eq!(count1, 1);

    // Second insert should increment count
    let (_id2, was_dup2, count2) = storage::persist(&pool, project_id, normalized.clone()).await?;
    assert_eq!(was_dup2, true);
    assert_eq!(count2, 2);

    // Verify ledger entries: there should be 2 entries for this error_id
    let ledger_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ledger WHERE error_id = $1")
        .bind(id1)
        .fetch_one(&pool)
        .await?;

    assert!(ledger_count >= 2);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_persist_transaction() -> Result<()> {
    // This test is ignored until a test DB is available
    Ok(())
