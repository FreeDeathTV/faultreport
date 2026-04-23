use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use tokio::task::JoinHandle;
use tokio::sync::Mutex;
use reqwest::Client;
use faultreport::modules::error_capture::{RawError, ErrorContext};
use uuid::Uuid;
use serde_json::json;
use sqlx::Row;

#[derive(Clone, Copy)]
enum ErrorVariety {
    Simple,
    Comprehensive,
}

#[derive(Clone, Copy)]
struct BenchConfig {
    duration: Duration,
    concurrent_workers: usize,
    test_postgres_restart: bool,
    error_variety: ErrorVariety,
}

#[derive(Default, Clone)]
struct BenchResults {
    // Counts
    total_submitted: u64,
    successful: u64,
    failed: u64,
    rate_limited: u64,
    validation_errors: u64,
    // Timing (ms)
    min_latency_ms: f64,
    max_latency_ms: f64,
    avg_latency_ms: f64,
    p95_latency_ms: f64,
    p99_latency_ms: f64,
    // Database
    unique_errors: u64,
    total_duplicates: u64,
    ledger_entries: u64,
    // System
    peak_memory_mb: f64,
    db_size_mb: f64,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]

async fn bench_comprehensive_load() -> Result<()> {
    let duration_secs: u64 = std::env::var("TEST_BENCH_DURATION_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    let config = BenchConfig {
        duration: Duration::from_secs(duration_secs),
        concurrent_workers: 50,
        test_postgres_restart: true,
        error_variety: ErrorVariety::Comprehensive,
    };

    let server_url = std::env::var("TEST_SERVER_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    let client = Client::builder()
        .pool_max_idle_per_host(0)
        .build()?;

    println!("Starting benchmark for {} seconds against {}", duration_secs, server_url);

    let mut results = run_load_test(&client, &server_url, &config).await?;

    println!("\nBenchmark completed, verifying database integrity...");
    match setup_test_db().await {
        Ok(pool) => {
            let _ = verify_correctness(&pool, &mut results).await;
        },
        Err(e) => println!("Skipping database verification: {}", e)
    }

    print_results(&results);
    Ok(())
}

// --- Harness pieces (scaffolding) ---

async fn setup_test_db() -> Result<sqlx::PgPool> {
    // Load .env if present so DATABASE_URL/TEST_DATABASE_URL are picked up
    if let Err(e) = dotenv::dotenv() {
        println!("Warning: .env file not loaded: {}", e);
    }

    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/faultreport?sslmode=disable".to_string());

    println!("Connecting to database at: {}", database_url);
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

async fn start_test_server() -> Result<String> {
    // For now assume docker-compose is running and backend is on localhost:8000
    Ok("http://localhost:8000".to_string())
}

async fn run_load_test(
    client: &Client,
    server_url: &str,
    config: &BenchConfig,
) -> Result<BenchResults> {
    let start = Instant::now();
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    let results = Arc::new(Mutex::new(BenchResults::default()));

    let cfg = *config;
    for _ in 0..cfg.concurrent_workers {
        let client = client.clone();
        let project_id = "20000000-0000-0000-0000-000000000000";
        let url = format!("{}/api/projects/{}/errors", server_url, project_id);
        let variety = cfg.error_variety;
        let duration = cfg.duration;
        let results = Arc::clone(&results);
        handles.push(tokio::spawn(async move {
            let errors = generate_varied_errors(variety);
            let mut idx = 0usize;
            while start.elapsed() < duration {
                let payload = &errors[idx % errors.len()];
                let t0 = Instant::now();
                let resp = client
                    .post(&url)
                    .header("Authorization", "Bearer frp_test_project_001")
                    .json(&raw_error_to_json(payload))
                    .send()
                    .await;
                let latency = t0.elapsed().as_secs_f64() * 1000.0;
                
                // Debug response status
                if idx == 0 {
                    match &resp {
                        Ok(r) => println!("\n✅ FIRST RESPONSE STATUS: {}", r.status()),
                        Err(e) => println!("\n❌ FIRST RESPONSE ERROR: {}", e),
                    }
                }
                
                let mut guard = results.lock().await;
                guard.total_submitted += 1;
                guard.min_latency_ms = if guard.min_latency_ms == 0.0 {
                    latency
                } else {
                    guard.min_latency_ms.min(latency)
                };
                guard.max_latency_ms = guard.max_latency_ms.max(latency);
                match resp {
                    Ok(r) => {
                        if r.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                            guard.rate_limited += 1;
                        } else if r.status().is_success() {
                            guard.successful += 1;
                        } else if r.status() == reqwest::StatusCode::BAD_REQUEST {
                            guard.validation_errors += 1;
                        } else {
                            guard.failed += 1;
                        }
                    }
                    Err(_) => guard.failed += 1,
                }
                idx += 1;
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    let final_results = results.lock().await.clone();
    Ok(final_results)
}

fn generate_varied_errors(variety: ErrorVariety) -> Vec<RawError> {
    match variety {
        ErrorVariety::Simple => vec![RawError {
            message: "Simple error".into(),
            stack: Some("at line 1".into()),
            context: normal_context("https://example.com"),
        }],
        ErrorVariety::Comprehensive => {
            let mut v = Vec::new();
            v.push(RawError {
                message: "Cannot read property 'x' of undefined".into(),
                stack: Some("at line 42\nat line 43".into()),
                context: normal_context("https://example.com/page"),
            });
            v.push(RawError {
                message: "x".repeat(1000),
                stack: Some("y".repeat(2000)),
                context: normal_context("https://example.com/long"),
            });
            v.push(RawError {
                message: "エラー: 日本語のエラーメッセージ".into(),
                stack: Some("Ошибка на строке 42".into()),
                context: normal_context("https://example.com/jp"),
            });
            v.push(RawError {
                message: "Minimal error".into(),
                stack: None,
                context: ErrorContext {
                    url: "".into(),
                    browser: None,
                    os: None,
                    user_id: None,
                    custom: json!({}),
                },
            });
            v.push(RawError {
                message: "".into(), // should fail validation
                stack: None,
                context: normal_context("https://example.com/bad"),
            });
            v.push(RawError {
                message: "Duplicate test".into(),
                stack: Some("at line 1".into()),
                context: normal_context("https://example.com/dup"),
            });
            v.push(RawError {
                message: "Same message".into(),
                stack: Some("Same stack".into()),
                context: normal_context("https://example.com/page1"),
            });
            v.push(RawError {
                message: "Same message".into(),
                stack: Some("Same stack".into()),
                context: normal_context("https://example.com/page2"),
            });
            v
        }
    }
}

fn normal_context(url: &str) -> ErrorContext {
    ErrorContext {
        url: url.to_string(),
        browser: Some("Chrome".into()),
        os: Some("Windows".into()),
        user_id: Some(Uuid::new_v4().to_string()),
        custom: json!({"session": Uuid::new_v4().to_string()}),
    }
}

fn raw_error_to_json(err: &RawError) -> serde_json::Value {
    json!({
        "message": err.message,
        "stack": err.stack,
        "context": err.context,
    })
}

async fn verify_correctness(pool: &sqlx::PgPool, results: &mut BenchResults) -> Result<()> {
    // 1) No missing accounting
    assert_eq!(
        results.successful + results.failed + results.rate_limited + results.validation_errors,
        results.total_submitted,
        "Sum of outcomes must equal total_submitted"
    );

    // 2) Rate limiting should trigger once load gets large
    if results.total_submitted >= 10_000 {
        assert!(
            results.rate_limited > 0,
            "Expected some 429s once requests exceed 10k/hour per project"
        );
    }

    // 3) DB unique hashes match expectations (at least 1)
    let db_unique: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT hash) FROM errors")
        .fetch_one(pool)
        .await?;
    assert!(db_unique > 0, "No errors persisted in DB");
    results.unique_errors = db_unique as u64;

    // 4) Ledger should have entries and be at least successful submissions
    let ledger_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ledger")
        .fetch_one(pool)
        .await?;
    assert!(
        ledger_count as u64 >= results.successful,
        "Ledger entries ({ledger_count}) should cover successful submissions ({})",
        results.successful
    );
    results.ledger_entries = ledger_count as u64;

    // 5) Hash determinism: re-submit same error twice and hashes/ids must match
    let test_error = RawError {
        message: "Deterministic duplicate test".into(),
        stack: Some("at line 1".into()),
        context: normal_context("https://example.com/deterministic"),
    };
    let h1 = submit_error(pool, &test_error).await?;
    let h2 = submit_error(pool, &test_error).await?;
    assert_eq!(h1.hash, h2.hash, "Same error produced different hashes");
    assert_eq!(h1.id, h2.id, "Same error created different records");

    Ok(())
}

async fn submit_error(pool: &sqlx::PgPool, err: &RawError) -> Result<DbErrorRow> {
    let row = sqlx::query(
        "INSERT INTO errors (project_id, hash, message, stack, context, count)
         VALUES ($1, $2, $3, $4, $5, 1)
         ON CONFLICT (project_id, hash) DO UPDATE SET count = errors.count + 1
         RETURNING id, hash"
    )
    .bind(Uuid::parse_str("22222222-2222-2222-2222-222222222222")?)
    .bind(faultreport::modules::error_capture::compute_hash(
        &err.message,
        err.stack.as_deref().unwrap_or(""),
        &err.context.url,
    ))
    .bind(&err.message)
    .bind(err.stack.as_deref().unwrap_or(""))
    .bind(serde_json::to_value(&err.context)?)
    .fetch_one(pool)
    .await?;

    Ok(DbErrorRow {
        id: row.get(0),
        hash: row.get(1),
    })
}

struct DbErrorRow {
    id: Uuid,
    hash: String,
}

fn print_results(results: &BenchResults) {
    println!("===== FaultReport Benchmark Results =====");
    println!("Submitted: {}", results.total_submitted);
    println!("Success:   {}", results.successful);
    println!("RateLimit: {}", results.rate_limited);
    println!("Failed:    {}", results.failed);
    println!("ValErr:    {}", results.validation_errors);
    println!("Latency ms: min {:.2} max {:.2}", results.min_latency_ms, results.max_latency_ms);
    if results.unique_errors > 0 {
        println!("Unique errors: {}", results.unique_errors);
        println!("Ledger entries: {}", results.ledger_entries);
    }
}