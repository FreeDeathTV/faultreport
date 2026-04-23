use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use std::result::Result;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::FaultReportError;

use crate::api_key::hash_api_key;

#[derive(Clone)]
pub struct ApiKeyCache {
    data: Arc<RwLock<HashMap<String, (Uuid, Uuid, DateTime<Utc>)>>>,
    ttl: Duration,
}

impl ApiKeyCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub async fn validate_key(&self, pool: &PgPool, key: &str) -> Result<(Uuid, Uuid), FaultReportError> {
        // 1. Fast path: check cache first
        {
            let cache = self.data.read();
            if let Some((project_id, user_id, expires_at)) = cache.get(key) {
                if Utc::now() < *expires_at {
                    return Ok((*project_id, *user_id));
                }
            }
        }

        // 2. Slow path: fetch all projects and verify by hashing with their salt.
        // This avoids relying on a DB-side searchable derived hash and is
        // acceptable for the small number of projects in tests/dev.
        let rows = sqlx::query(
            "SELECT id, created_by_user_id, api_key_salt FROM projects WHERE revoked_at IS NULL"
        )
        .fetch_all(pool)
        .await?;

        for r in rows {
            let project_id: Uuid = r.get::<Uuid, _>("id");
            let user_id: Uuid = r.get::<Uuid, _>("created_by_user_id");
            let salt: String = r.get::<String, _>("api_key_salt");

            let computed_hash = hash_api_key(key, &salt);

            let valid: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM projects WHERE api_key_hash = $1 AND revoked_at IS NULL)"
            )
            .bind(&computed_hash)
            .fetch_one(pool)
            .await?;

            if valid {
                // Compute expiry without using `from_std` to avoid cross-error conversions
                let expires_at = Utc::now() + chrono::Duration::seconds(self.ttl.as_secs() as i64);
                let mut cache = self.data.write();

                if cache.len() >= 1000 {
                    if let Some((oldest_key, _)) = cache.iter()
                        .min_by_key(|(_, (_, _, expires))| *expires)
                        .map(|(k, _)| (k.clone(), ())) {
                        cache.remove(&oldest_key);
                    }
                }

                cache.insert(key.to_string(), (project_id, user_id, expires_at));
                return Ok((project_id, user_id));
            }
        }

        Err(FaultReportError::InvalidApiKey)
    }

    /// Clear expired entries from cache (call periodically if needed)
    pub fn purge_expired(&self) {
        let now = Utc::now();
        let mut cache = self.data.write();
        cache.retain(|_, (_, _, expires_at)| now < *expires_at);
    }
}
