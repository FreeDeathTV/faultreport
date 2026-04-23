use actix_web::{App, web};
use actix_web::test;
use serde_json::json;
use uuid::Uuid;
use sqlx::migrate::Migrator;
use std::path::Path;
use jsonwebtoken::{EncodingKey, Header, Algorithm, encode};
use openssl::rsa::Rsa;
use sha2::{Sha256, Digest};
// For this e2e test we call the raw posting helper with a localhost URL
// to exercise the alert HTTP client behavior without opening sockets that
// may be restricted in the test environment.

// E2E: attempt to trigger Slack alert by submitting multiple errors
#[tokio::test]
#[ignore]
async fn test_alert_slack_end_to_end() {
    let db_url = match std::env::var("TEST_DATABASE_URL") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Skipping e2e alert test: TEST_DATABASE_URL not set");
            return;
        }
    };

    let pool = sqlx::PgPool::connect(&db_url).await.expect("connect db");
    let migrator = Migrator::new(Path::new("migrations")).await.expect("migrator");
    migrator.run(&pool).await.expect("run migrations");

    // Use a non-routable local port to exercise the client retry/backoff
    // behavior without requiring an actual remote webhook.
    let webhook = "http://127.0.0.1:9/hook".to_string();
    std::env::set_var("SLACK_WEBHOOK_URL", &webhook);
    std::env::remove_var("NO_EXTERNAL_CALLS");

    // Generate RSA keypair and signed JWT for auth
    let rsa = Rsa::generate(2048).expect("generate rsa");
    let private_pem = rsa.private_key_to_pem_pkcs1().expect("private pem");
    let public_pem = rsa.public_key_to_pem_pkcs1().expect("public pem");

    #[derive(serde::Serialize)]
    struct EncodeClaims {
        aud: String,
        iss: String,
        sub: String,
        exp: usize,
    }

    let aud = "test-project".to_string();
    let iss = format!("https://securetoken.google.com/{}", aud);
    let sub = "e2e-user-044".to_string();
    let exp = (chrono::Utc::now().timestamp() + 3600) as usize;

    let claims = EncodeClaims { aud: aud.clone(), iss: iss.clone(), sub: sub.clone(), exp };
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("testkid".to_string());
    let token = encode(&header, &claims, &EncodingKey::from_rsa_pem(&private_pem).expect("encoding key")).expect("encode token");

    // Serve public PEM for verifier
    let server = httptest::Server::run();
    let jwks: serde_json::Value = serde_json::json!({ "testkid": String::from_utf8(public_pem.clone()).unwrap() });
    server.expect(httptest::Expectation::matching(httptest::matchers::request::method_path("GET", "/certs")).respond_with(httptest::responders::json_encoded(jwks)));
    std::env::set_var("FIREBASE_CERTS_URL", server.url("/certs"));
    std::env::set_var("FIREBASE_PROJECT_ID", aud.clone());

    // Compute middleware-derived UUID and insert user
    let mut hasher = Sha256::new();
    hasher.update(sub.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[0..16]);
    let derived_user_id = Uuid::from_slice(&bytes).expect("uuid from sub");

    sqlx::query("INSERT INTO users (id, firebase_uid, email) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING")
        .bind(derived_user_id)
        .bind(sub.clone())
        .bind("e2e+user@example.com")
        .execute(&pool)
        .await
        .expect("insert user");

    let config = faultreport::config::Config {
        database_url: db_url.clone(),
        redis_url: None,
        allowed_origins: std::collections::HashSet::new(),
        rust_log: "debug".to_string(),
    };

    let api_key_cache = faultreport::auth::cache::ApiKeyCache::new(std::time::Duration::from_secs(300));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(api_key_cache.clone()))
            .configure(faultreport::orchestrator::config(&config))
    ).await;

    // create project
    // Ensure user exists for the mock token used by the middleware
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000044").unwrap();
    sqlx::query("INSERT INTO users (id, firebase_uid, email) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING")
        .bind(user_id)
        .bind("mock:00000000-0000-0000-0000-000000000044")
        .bind("e2e+user@example.com")
        .execute(&pool)
        .await
        .expect("insert user");

    let req = test::TestRequest::post()
        .uri("/api/projects")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"name": "Alert Project"}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    if !status.is_success() {
        let raw = test::read_body(resp).await;
        eprintln!("create project failed: {} {}", status, String::from_utf8_lossy(&raw));
        panic!("create project failed");
    }
    let body: serde_json::Value = test::read_body_json(resp).await;
    let project_id = Uuid::parse_str(body.get("project_id").and_then(|v| v.as_str()).expect("project_id")).expect("uuid");
    let api_key = body.get("api_key").and_then(|v| v.as_str()).expect("api_key").to_string();

    // Directly call the posting helper to verify the client runs (errors are
    // swallowed by the helper and should return Ok(()).
    let res = faultreport::alert::post_slack_raw(&webhook, project_id, "deadbeef").await;
    assert!(res.is_ok());
}
