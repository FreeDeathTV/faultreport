use actix_web::{App, web};
use actix_web::test;
use serde_json::json;
use uuid::Uuid;
use sqlx::migrate::Migrator;
use sqlx::Row;
use std::path::Path;
use jsonwebtoken::{EncodingKey, Header, Algorithm, encode};
use openssl::rsa::Rsa;
use sha2::{Sha256, Digest};

// E2E: Submit the same error 100 times and verify deterministic hash and count
#[tokio::test]
#[ignore]
async fn test_hash_determinism() {
    let db_url = match std::env::var("TEST_DATABASE_URL") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Skipping determinism test: TEST_DATABASE_URL not set");
            return;
        }
    };

    let pool = sqlx::PgPool::connect(&db_url).await.expect("connect db");
    let migrator = Migrator::new(Path::new("migrations")).await.expect("migrator");
    migrator.run(&pool).await.expect("run migrations");

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

    // create user + project
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
    let sub = "e2e-user-099".to_string();
    let exp = (chrono::Utc::now().timestamp() + 3600) as usize;

    let claims = EncodeClaims { aud: aud.clone(), iss: iss.clone(), sub: sub.clone(), exp };
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("testkid".to_string());
    let token = encode(&header, &claims, &EncodingKey::from_rsa_pem(&private_pem).expect("encoding key")).expect("encode token");

    // Serve public PEM via a local test server for the verifier
    let server = httptest::Server::run();
    let jwks: serde_json::Value = serde_json::json!({ "testkid": String::from_utf8(public_pem.clone()).unwrap() });
    server.expect(httptest::Expectation::matching(httptest::matchers::request::method_path("GET", "/certs")).respond_with(httptest::responders::json_encoded(jwks)));

    std::env::set_var("FIREBASE_CERTS_URL", server.url("/certs"));
    std::env::set_var("FIREBASE_PROJECT_ID", aud.clone());

    // Compute the middleware-derived UUID from the Firebase UID (sha256 -> first 16 bytes)
    let mut hasher = Sha256::new();
    hasher.update(sub.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[0..16]);
    let derived_user_id = Uuid::from_slice(&bytes).expect("uuid from sub");

    // Ensure user exists for the derived UID
    sqlx::query("INSERT INTO users (id, firebase_uid, email) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING")
        .bind(derived_user_id)
        .bind(sub.clone())
        .bind("e2e+user@example.com")
        .execute(&pool)
        .await
        .expect("insert user");

    let req = test::TestRequest::post()
        .uri("/api/projects")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"name": "Determinism Project"}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    let project_id = Uuid::parse_str(body.get("project_id").and_then(|v| v.as_str()).expect("project_id")).expect("uuid");
    let api_key = body.get("api_key").and_then(|v| v.as_str()).expect("api_key").to_string();

    let raw_error = json!({
        "message": "Determinism test",
        "stack": "frame1\nframe2\nframe3",
        "context": { "url": "https://example.com" }
    });

    let submit_path = format!("/api/projects/{}/errors", project_id);

    // submit same error 100 times
    for _ in 0..100u32 {
        let r = test::TestRequest::post()
            .uri(&submit_path)
            .insert_header(("Authorization", format!("Bearer {}", api_key)))
            .set_json(&raw_error)
            .to_request();
        let resp = test::call_service(&app, r).await;
        assert_eq!(resp.status().as_u16(), 201);
    }

    // verify errors table has one row with count == 100
    let row = sqlx::query("SELECT hash, count FROM errors WHERE project_id = $1")
        .bind(project_id)
        .fetch_one(&pool)
        .await
        .expect("select error");
    let count: i64 = row.get::<i64, _>("count");
    assert_eq!(count, 100);

    let hash: String = row.get::<String, _>("hash");
    assert!(!hash.is_empty());
}
