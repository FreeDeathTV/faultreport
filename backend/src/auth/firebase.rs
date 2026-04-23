use anyhow::Result;
use crate::error::FaultReportError;
use reqwest::Client;
use serde::Deserialize;
use std::env;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

// This verifier fetches public certs from a Firebase certs endpoint (x509 map)
// and verifies JWTs locally using the RSA public key. The endpoint is
// configurable via `FIREBASE_CERTS_URL` to allow deterministic testing.

#[derive(Deserialize)]
struct Claims {
    aud: String,
    iss: String,
    sub: String,
    exp: usize,
}

static CERT_CACHE: Lazy<Mutex<(HashMap<String, String>, Instant)>> = Lazy::new(|| Mutex::new((HashMap::new(), Instant::now())));

async fn fetch_certs() -> Result<HashMap<String, String>, FaultReportError> {
    let certs_url = env::var("FIREBASE_CERTS_URL").unwrap_or_else(|_| "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com".to_string());

    // Simple cache with 1 minute TTL
    {
        let cache = CERT_CACHE.lock();
        if Instant::now() < cache.1 {
            return Ok(cache.0.clone());
        }
    }

    let client = Client::new();
    let res = client.get(&certs_url).send().await.map_err(|_| FaultReportError::Unauthorized)?;
    if !res.status().is_success() {
        return Err(FaultReportError::Unauthorized);
    }

    let map: HashMap<String, String> = res.json().await.map_err(|_| FaultReportError::Unauthorized)?;

    // update cache
    {
        let mut cache = CERT_CACHE.lock();
        cache.0 = map.clone();
        cache.1 = Instant::now() + Duration::from_secs(60);
    }

    Ok(map)
}

pub async fn verify_firebase_token(token: &str) -> Result<String, FaultReportError> {
    let certs = fetch_certs().await?;

    // Decode header to find kid
    let header = jsonwebtoken::decode_header(token).map_err(|_| FaultReportError::Unauthorized)?;
    let kid = header.kid.ok_or(FaultReportError::Unauthorized)?;

    let pem = certs.get(&kid).ok_or(FaultReportError::Unauthorized)?;

    let decoding_key = DecodingKey::from_rsa_pem(pem.as_bytes()).map_err(|_| FaultReportError::Unauthorized)?;

    // FIREBASE_PROJECT_ID is mandatory for security
    let project_id = env::var("FIREBASE_PROJECT_ID")
        .map_err(|_| FaultReportError::Unauthorized)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[&project_id]);
    validation.set_issuer(&[&format!("https://securetoken.google.com/{}", project_id)]);
    validation.validate_exp = true;
    validation.required_spec_claims.insert("sub".to_string());

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|_| FaultReportError::Unauthorized)?;

    Ok(token_data.claims.sub)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httptest::{Server, Expectation, matchers::*, responders::*};

    #[tokio::test]
    async fn test_verify_firebase_token_with_signed_jwt() {
        use jsonwebtoken::{EncodingKey, Header, Algorithm, encode};
        use openssl::rsa::Rsa;
        use chrono::Utc;

        // Generate RSA keypair
        let rsa = Rsa::generate(2048).expect("generate rsa");
        let private_pem = rsa.private_key_to_pem_pkcs1().expect("private pem");
        let public_pem = rsa.public_key_to_pem_pkcs1().expect("public pem");

        // Create claims
        #[derive(serde::Serialize)]
        struct EncodeClaims {
            aud: String,
            iss: String,
            sub: String,
            exp: usize,
        }

        let aud = "test-project".to_string();
        let iss = format!("https://securetoken.google.com/{}", aud);
        let sub = "test-user-123".to_string();
        let exp = (Utc::now().timestamp() + 3600) as usize;

        let claims = EncodeClaims { aud: aud.clone(), iss: iss.clone(), sub: sub.clone(), exp };

        // Header with kid
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some("testkid".to_string());

        // Sign token using private PEM
        let token = encode(&header, &claims, &EncodingKey::from_rsa_pem(&private_pem).expect("encoding key")).expect("encode token");

        // Serve public PEM via test server as the certs map
        let server = Server::run();
        let jwks: serde_json::Value = serde_json::json!({ "testkid": String::from_utf8(public_pem.clone()).unwrap() });
        server.expect(
            Expectation::matching(request::method_path("GET", "/certs")).respond_with(json_encoded(jwks))
        );

        std::env::set_var("FIREBASE_CERTS_URL", server.url("/certs"));
        std::env::set_var("FIREBASE_PROJECT_ID", aud);

        // Verify token
        let res = verify_firebase_token(&token).await.expect("verify token");
        assert_eq!(res, sub);
    }

    #[tokio::test]
    async fn test_verify_firebase_token_rejects_missing_project_id() {
        // Clear any existing FIREBASE_PROJECT_ID
        std::env::remove_var("FIREBASE_PROJECT_ID");

        let result = verify_firebase_token("invalid.token.here").await;
        assert!(result.is_err());
    }
}
