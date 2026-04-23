use actix_web::HttpRequest;
use crate::error::FaultReportError;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use crate::auth::firebase;

/// Verify Firebase auth token.
///
/// Production behavior:
/// - Requires a valid Firebase JWT in the `Authorization: Bearer <token>` header.
/// - Validates the token signature, audience, issuer, and expiration.
/// - Derives a stable UUID from the Firebase UID for internal use.
pub async fn require_firebase_auth(req: &HttpRequest) -> Result<Uuid, FaultReportError> {
    let token = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(FaultReportError::Unauthorized)?;

    // Verify token with Firebase.
    let uid = firebase::verify_firebase_token(token).await?;
    let digest = Sha256::digest(uid.as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[0..16]);
    Ok(Uuid::from_slice(&bytes).map_err(|_| FaultReportError::Unauthorized)?)
}

#[cfg(test)]
pub mod test_helpers {
    use super::*;

    /// Test helper: bypass Firebase auth for deterministic local testing.
    /// Only available when compiled with `#[cfg(test)]`.
    pub async fn require_firebase_auth_mock(req: &HttpRequest) -> Result<Uuid, FaultReportError> {
        if env::var("DISABLE_FIREBASE_AUTH").ok().map(|v| {
            let lv = v.to_ascii_lowercase();
            lv == "1" || lv == "true" || lv == "yes"
        }).unwrap_or(false) {
            if let Ok(u) = env::var("MOCK_USER_ID") {
                return Uuid::parse_str(&u).map_err(|_| FaultReportError::Unauthorized);
            }
            return Ok(Uuid::nil());
        }

        let token = req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or(FaultReportError::Unauthorized)?;

        // Support `mock:<uuid>` tokens for deterministic local testing
        if let Some(s) = token.strip_prefix("mock:") {
            return Uuid::parse_str(s).map_err(|_| FaultReportError::Unauthorized);
        }

        let uid = firebase::verify_firebase_token(token).await?;
        let digest = Sha256::digest(uid.as_bytes());
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&digest[0..16]);
        Ok(Uuid::from_slice(&bytes).map_err(|_| FaultReportError::Unauthorized)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;
    use sha2::{Sha256, Digest};

    #[tokio::test]
    async fn test_require_firebase_auth_rejects_mock_prefix_in_prod() {
        let req = TestRequest::default()
            .insert_header(("Authorization", "Bearer mock:00000000-0000-0000-0000-000000000002"))
            .to_http_request();
        let got = require_firebase_auth(&req).await;
        assert!(got.is_err());
    }

    #[tokio::test]
    async fn test_require_firebase_auth_rejects_missing_header() {
        let req = TestRequest::default().to_http_request();
        let got = require_firebase_auth(&req).await;
        assert!(got.is_err());
    }
}
