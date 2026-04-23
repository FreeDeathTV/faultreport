use anyhow::Result;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde_json;
use chrono::Utc;
use crate::modules::error_capture::NormalizedError;
use serde::Serialize;
use chrono::Timelike;

pub async fn persist(pool: &PgPool, project_id: Uuid, error: NormalizedError) -> Result<(Uuid, bool, i64)> {
    // Perform all write operations in a single transaction to avoid
    // partial writes and race conditions under concurrent submissions.
    // Acquire a dedicated connection from the pool and start an explicit
    // transaction using raw BEGIN/COMMIT so we can use the underlying
    // connection as the executor (compatible across sqlx versions).
    let mut conn = pool.acquire().await?;
    sqlx::query("BEGIN").execute(&mut *conn).await?;

    // Idempotent: check existing by hash with FOR UPDATE to lock the row
    let existing: Option<(Uuid, i64)> = sqlx::query_as(
        "SELECT id, count FROM errors WHERE project_id = $1 AND hash = $2 FOR UPDATE"
    )
    .bind(project_id)
    .bind(&error.hash)
    .fetch_optional(&mut *conn)
    .await?;

    if let Some((id, count)) = existing {
        // Duplicate: increment
        let new_count = count + 1;
        sqlx::query(
            "UPDATE errors SET count = $1, last_seen_at = NOW() WHERE id = $2"
        )
        .bind(new_count)
        .bind(id)
        .execute(&mut *conn)
        .await?;

        // Ledger duplicate
        let ledger_data = serde_json::json!({"old_count": count, "new_count": new_count});
        sqlx::query(
            "INSERT INTO ledger (project_id, error_id, event_type, data) VALUES ($1, $2, $3, $4)"
        )
        .bind(project_id)
        .bind(id)
        .bind("duplicate")
        .bind(ledger_data)
        .execute(&mut *conn)
        .await?;

        sqlx::query("COMMIT").execute(&mut *conn).await?;
        return Ok((id, true, new_count));
    }

    // New error path
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO errors (id, project_id, hash, message, stack, context, first_seen_at, last_seen_at, count) 
         VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW(), 1)"
    )
    .bind(id)
    .bind(project_id)
    .bind(&error.hash)
    .bind(&error.message)
    .bind(&error.stack)
    .bind(serde_json::to_value(&error.context)?)
    .execute(&mut *conn)
    .await?;

    // Ledger new
    sqlx::query(
        "INSERT INTO ledger (project_id, error_id, event_type) VALUES ($1, $2, $3)"
    )
    .bind(project_id)
    .bind(id)
    .bind("new_error")

    .execute(&mut *conn)
    .await?;

    sqlx::query("COMMIT").execute(&mut *conn).await?;

    Ok((id, false, 1))
}

pub async fn check_rate_limit(pool: &PgPool, project_id: Uuid) -> Result<bool> {
    // For now use database-backed rate limiting fallback only.
    check_rate_limit_fallback(pool, project_id).await
}

async fn check_rate_limit_fallback(pool: &PgPool, project_id: Uuid) -> Result<bool> {
    let now = Utc::now();
    let hour_window = now.date_naive().and_hms_opt(now.hour(), 0, 0).unwrap();

    let count: i64 = sqlx::query_scalar(
        "SELECT COALESCE(error_count, 0) FROM rate_limit_tracker
         WHERE project_id = $1 AND hour_window = $2"
    )
    .bind(project_id)
    .bind(hour_window)
    .fetch_optional(pool)
    .await?
    .unwrap_or(0);

    Ok(count < 10000)
}

pub async fn increment_rate_limit(pool: &PgPool, project_id: Uuid) -> Result<()> {
    // Use database-backed fallback implementation for now.
    increment_rate_limit_fallback(pool, project_id).await
}

async fn increment_rate_limit_fallback(pool: &PgPool, project_id: Uuid) -> Result<()> {
    let now = Utc::now();
    let hour_window = now.date_naive().and_hms_opt(now.hour(), 0, 0).unwrap();

    sqlx::query(
        "INSERT INTO rate_limit_tracker (project_id, hour_window, error_count, updated_at)
         VALUES ($1, $2, 1, NOW())
         ON CONFLICT (project_id, hour_window)
         DO UPDATE SET error_count = error_count + 1, updated_at = NOW()"
    )
    .bind(project_id)
    .bind(hour_window)
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(Serialize)]
pub struct ErrorRecord {
    pub id: Uuid,
    pub hash: String,
    pub message: String,
    pub stack: String,
    pub context: serde_json::Value,
    pub count: i64,
    pub first_seen_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_errors(pool: &PgPool, project_id: Uuid, limit: i64) -> Result<Vec<ErrorRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT id, hash, message, stack, context, count, first_seen_at, last_seen_at
        FROM errors
        WHERE project_id = $1
        ORDER BY last_seen_at DESC
        LIMIT $2
        "#
    )
    .bind(project_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|r| ErrorRecord {
            id: r.get("id"),
            hash: r.get("hash"),
            message: r.get("message"),
            stack: r.try_get("stack").unwrap_or_default(),
            context: r.try_get("context").unwrap_or_else(|_| serde_json::json!({})),
            count: r.try_get("count").unwrap_or(0),
            first_seen_at: r.get("first_seen_at"),
            last_seen_at: r.get("last_seen_at"),
        })
        .collect();

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_error_record_serialize_contains_fields() {
        let now = Utc::now();
        let rec = ErrorRecord {
            id: Uuid::nil(),
            hash: "h123".to_string(),
            message: "msg".to_string(),
            stack: "".to_string(),
            context: serde_json::json!({"k": "v"}),
            count: 1,
            first_seen_at: now,
            last_seen_at: now,
        };

        let s = serde_json::to_string(&rec).expect("serialize");
        assert!(s.contains("\"hash\":\"h123\""));
        assert!(s.contains("\"message\":\"msg\""));
    }
}