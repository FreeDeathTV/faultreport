use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FaultReportError {
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid API key")]
    InvalidApiKey,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Anyhow: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl ResponseError for FaultReportError {
    fn error_response(&self) -> HttpResponse {
        match self {
            FaultReportError::Validation(msg) => {
                HttpResponse::BadRequest().json(serde_json::json!({"error": msg}))
            }
            FaultReportError::RateLimitExceeded => {
                HttpResponse::TooManyRequests().json(serde_json::json!({"error": "Rate limit exceeded"}))
            }
            FaultReportError::InvalidApiKey => {
                HttpResponse::Unauthorized().json(serde_json::json!({"error": "Invalid API key"}))
            }
            FaultReportError::Unauthorized => {
                HttpResponse::Unauthorized().json(serde_json::json!({"error": "Unauthorized"}))
            }
            _ => {
                // Log detailed error server-side; return generic message to client
                tracing::error!("Internal server error: {}", self);
                HttpResponse::InternalServerError().json(serde_json::json!({"error": "Internal server error"}))
            }
        }
    }
}
