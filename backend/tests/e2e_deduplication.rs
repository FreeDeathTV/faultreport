use actix_web::{App, web};
use actix_web::test;
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;
use sqlx::migrate::Migrator;
use std::path::Path;
use jsonwebtoken::{EncodingKey, Header, Algorithm, encode};
use openssl::rsa::Rsa;
use sha2::{Sha256, Digest};

// E2E: submitting same error twice should mark second as duplicate and count=2
#[tokio::test]
#[tokio::test]
#[ignore]
async fn test_deduplication_end_to_end() {
    let db_url = match std::env::var("TEST_DATABASE_URL") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Skipping e2e dedup test: TEST_DATABASE_URL not set");
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

    // Generate signed JWT and insert corresponding user
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
    let sub = "e2e-user-022".to_string();
    let exp = (chrono::Utc::now().timestamp() + 3600) as usize;

    let claims = EncodeClaims { aud: aud.clone(), iss: iss.clone(), sub: sub.clone(), exp };
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some("testkid".to_string());
    let token = encode(&header, &claims, &EncodingKey::from_rsa_pem(&private_pem).expect("encoding key")).expect("encode token");

    let server = httptest::Server::run();
    let jwks: serde_json::Value = serde_json::json!({ "testkid": String::from_utf8(public_pem.clone()).unwrap() });
    server.expect(httptest::Expectation::matching(httptest::matchers::request::method_path("GET", "/certs")).respond_with(httptest::responders::json_encoded(jwks)));
    std::env::set_var("FIREBASE_CERTS_URL", server.url("/certs"));
    std::env::set_var("FIREBASE_PROJECT_ID", aud.clone());

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

    let req = test::TestRequest::post()
        .uri("/api/projects")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"name": "Dedup Project"}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    let project_id = Uuid::parse_str(body.get("project_id").and_then(|v| v.as_str()).expect("project_id")).expect("uuid");
    let api_key = body.get("api_key").and_then(|v| v.as_str()).expect("api_key").to_string();
    eprintln!("created api_key: {}", api_key);
    // Debug: read stored salt/hash and verify computed hash locally
    let row = sqlx::query("SELECT api_key_hash, api_key_salt FROM projects WHERE id = $1")
        .bind(project_id)
        .fetch_one(&pool)
        .await
        .expect("select project");
    let stored_hash: String = row.get::<String, _>("api_key_hash");
    let salt: String = row.get::<String, _>("api_key_salt");
    let computed = faultreport::modules::projects::hash_api_key(&api_key, &salt);
    eprintln!("stored_hash: {}", stored_hash);
    eprintln!("computed_hash: {}", computed);

    let raw_error = json!({
        "message": "Dedup test",
        "stack": "frame1\nframe2",
        "context": { "url": "https://example.com" }
    });

    let submit_path = format!("/api/projects/{}/errors", project_id);

    // first submit
    let r1 = test::TestRequest::post()
        .uri(&submit_path)
        .insert_header(("Authorization", format!("Bearer {}", api_key)))
        .set_json(&raw_error)
        .to_request();
    let resp1 = test::call_service(&app, r1).await;
    let status1 = resp1.status();
    if status1.as_u16() != 201 {
        let raw = test::read_body(resp1).await;
        eprintln!("first submit failed: {} {}", status1, String::from_utf8_lossy(&raw));
        panic!("first submit failed");
    }

    // second submit (same payload)
    let r2 = test::TestRequest::post()
        .uri(&submit_path)
        .insert_header(("Authorization", format!("Bearer {}", api_key)))
        .set_json(&raw_error)
        .to_request();
    let resp2 = test::call_service(&app, r2).await;
    let status2 = resp2.status();
    if status2.as_u16() != 201 {
        let raw = test::read_body(resp2).await;
        eprintln!("second submit failed: {} {}", status2, String::from_utf8_lossy(&raw));
        panic!("second submit failed");
    }
    let j: serde_json::Value = test::read_body_json(resp2).await;

    // Second response should indicate duplicate and count == 2
    assert_eq!(j.get("was_duplicate").and_then(|v| v.as_bool()).unwrap_or(false), true);
    assert_eq!(j.get("count").and_then(|v| v.as_i64()).unwrap_or(0), 2);
}
