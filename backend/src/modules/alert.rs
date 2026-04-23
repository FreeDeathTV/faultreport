use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};
use serde_json::json;
use reqwest::Client;
use std::env;
use std::time::Duration as StdDuration;
use tokio::time::sleep;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::HashMap;
use chrono::Utc as ChronoUtc;

pub async fn should_alert(pool: &PgPool, project_id: Uuid, error_hash: &str) -> Result<bool> {
    let five_min_ago = Utc::now() - Duration::minutes(5);
let recent_alert = sqlx::query("SELECT 1 FROM alert_dedup WHERE project_id = $1 AND error_hash = $2 AND last_alert_at > $3")
    .bind(project_id)
    .bind(error_hash)
    .bind(five_min_ago)
    .fetch_optional(pool)
    .await?;

    Ok(recent_alert.is_none())
}

pub async fn record_alert(pool: &PgPool, project_id: Uuid, error_hash: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO alert_dedup (project_id, error_hash, last_alert_at) VALUES ($1, $2, NOW())
         ON CONFLICT (project_id, error_hash) DO UPDATE SET last_alert_at = NOW()"
    )
    .bind(project_id)
    .bind(error_hash)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn post_slack(pool: &PgPool, project_id: Uuid, error_hash: &str) -> Result<()> {
    // If CI or runtime disables external calls, skip posting.
    if let Ok(flag) = env::var("NO_EXTERNAL_CALLS") {
        let f = flag.to_ascii_lowercase();
        if f == "1" || f == "true" || f == "yes" {
            return Ok(());
        }
    }

    // If no webhook configured, skip silently.
    let webhook = match env::var("SLACK_WEBHOOK_URL") {
        Ok(u) if !u.is_empty() => u,
        _ => return Ok(()),
    };

    // Delegate to a small testable helper that posts to the webhook URL.
    post_slack_raw(&webhook, project_id, error_hash).await
}

/// Testable helper that posts a Slack-format JSON payload to a webhook URL.
/// This is public so integration tests can call it directly without requiring a DB.
pub async fn post_slack_raw(webhook: &str, project_id: Uuid, error_hash: &str) -> Result<()> {
    // Use AlertClient which implements a small retry/backoff policy.
    let client = AlertClient::from_env();
    client.post(webhook, project_id, error_hash).await
}

/// Small HTTP alerting client with retry/backoff. Best-effort: failures are
/// logged and do not propagate to caller as errors.
pub struct AlertClient {
    client: Client,
    max_retries: usize,
    backoff_base_ms: u64,
}

impl AlertClient {
    pub fn from_env() -> Self {
        let max_retries = env::var("ALERT_MAX_RETRIES").ok().and_then(|s| s.parse().ok()).unwrap_or(3usize);
        let backoff_base_ms = env::var("ALERT_BACKOFF_MS").ok().and_then(|s| s.parse().ok()).unwrap_or(200u64);
        Self {
            client: Client::new(),
            max_retries,
            backoff_base_ms,
        }
    }

    pub async fn post(&self, webhook: &str, project_id: Uuid, error_hash: &str) -> Result<()> {
        let text = format!("FaultReport: Spike detected for project {} (error {})", project_id, error_hash);
        let payload = json!({ "text": text });

        // Check circuit breaker state for this webhook
        if CircuitBreaker::is_open(webhook) {
            return Ok(());
        }

        for attempt in 0..=self.max_retries {
            let res = self.client.post(webhook).json(&payload).send().await;
            match res {
                Ok(resp) if resp.status().is_success() => return Ok(()),
                _ => {
                    // record failure
                    CircuitBreaker::record_failure(webhook);
                    if attempt >= self.max_retries {
                        // give up silently — alerts are best-effort
                        return Ok(());
                    }
                    // exponential backoff
                    let backoff = self.backoff_base_ms.saturating_mul(2u64.pow(attempt as u32));
                    sleep(StdDuration::from_millis(backoff)).await;
                }
            }
        }

        Ok(())
    }
}

// Simple in-memory circuit breaker keyed by webhook URL.
struct CircuitState {
    failures: u32,
    open_until_ts: i64, // unix timestamp seconds
}

static CIRCUIT_BREAKER: Lazy<Mutex<HashMap<String, CircuitState>>> = Lazy::new(|| Mutex::new(HashMap::new()));

struct CircuitBreaker;

impl CircuitBreaker {
    // if open, return true
    fn is_open(key: &str) -> bool {
        let map = CIRCUIT_BREAKER.lock();
        if let Some(state) = map.get(key) {
            if state.open_until_ts > ChronoUtc::now().timestamp() {
                return true;
            }
        }
        false
    }

    fn record_failure(key: &str) {
        let mut map = CIRCUIT_BREAKER.lock();
        let entry = map.entry(key.to_string()).or_insert(CircuitState { failures: 0, open_until_ts: 0 });
        entry.failures = entry.failures.saturating_add(1);
        // If failures exceed threshold, open the circuit for a cooldown period
        let threshold = std::env::var("ALERT_CB_THRESHOLD").ok().and_then(|s| s.parse().ok()).unwrap_or(5u32);
        let cooldown_secs = std::env::var("ALERT_CB_COOLDOWN").ok().and_then(|s| s.parse().ok()).unwrap_or(300i64);
        if entry.failures >= threshold {
            entry.open_until_ts = ChronoUtc::now().timestamp() + cooldown_secs;
            // reset failures after tripping
            entry.failures = 0;
        }
    }

    #[allow(dead_code)]
    fn record_success(key: &str) {
        let mut map = CIRCUIT_BREAKER.lock();
        map.remove(key);
    }
}

pub async fn check_spike(pool: &PgPool, project_id: Uuid, error_hash: &str) -> Result<bool> {
    let five_min_ago = Utc::now() - Duration::minutes(5);
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM errors WHERE project_id = $1 AND hash = $2 AND last_seen_at > $3"
    )
    .bind(project_id)
    .bind(error_hash)
    .bind(five_min_ago)
    .fetch_one(pool)
    .await?;

    Ok(count > 10)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    // Unit tests that do not require a database. Integration tests which need
    // a Postgres instance should be placed under `tests/` and run with
    // `-- --ignored` when a test DB is available.

    #[test]
    fn test_alert_client_from_env() {
        env::set_var("ALERT_MAX_RETRIES", "2");
        env::set_var("ALERT_BACKOFF_MS", "100");

        let c = AlertClient::from_env();
        assert_eq!(c.max_retries, 2usize);
        assert_eq!(c.backoff_base_ms, 100u64);
    }

    #[test]
    fn test_circuit_breaker_trips_and_resets() {
        // Set low threshold and cooldown so test is deterministic
        env::set_var("ALERT_CB_THRESHOLD", "2");
        env::set_var("ALERT_CB_COOLDOWN", "60");

        let key = "__test_webhook__";
        // Ensure clean state
        CircuitBreaker::record_success(key);
        assert!(!CircuitBreaker::is_open(key));

        // Trigger failures to trip the breaker
        CircuitBreaker::record_failure(key);
        CircuitBreaker::record_failure(key);
        // After threshold reached, breaker should be open
        assert!(CircuitBreaker::is_open(key));

        // Reset
        CircuitBreaker::record_success(key);
        assert!(!CircuitBreaker::is_open(key));
    }

    #[tokio::test]
    async fn test_post_slack_raw_retries() {
        use httptest::{Server, Expectation, matchers::*, responders::*};
        use uuid::Uuid;

        // Start a local test HTTP server to act as the Slack webhook sink.
        let server = Server::run();

        // Simulate two transient failures then a success to exercise retry logic.
        server.expect(
            Expectation::matching(request::method_path("POST", "/hook")).times(1)
                .respond_with(status_code(500))
        );
        server.expect(
            Expectation::matching(request::method_path("POST", "/hook")).times(1)
                .respond_with(status_code(500))
        );
        server.expect(
            Expectation::matching(request::method_path("POST", "/hook")).times(1)
                .respond_with(status_code(200))
        );

        let webhook = server.url("/hook");
        let project_id = Uuid::new_v4();

        // Ensure NO_EXTERNAL_CALLS is not set for this test
        std::env::remove_var("NO_EXTERNAL_CALLS");

        let res = post_slack_raw(&webhook, project_id, "deadbeef").await;
        assert!(res.is_ok());
    }
}

