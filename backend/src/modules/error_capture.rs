use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use hex;
use crate::error::FaultReportError;
// Removed unused Uuid

#[derive(Deserialize)]
pub struct RawError {
    pub message: String,
    pub stack: Option<String>,
    pub context: ErrorContext,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ErrorContext {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub browser: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(flatten)]
    pub custom: serde_json::Value,
}

#[derive(Serialize, Clone)]
pub struct NormalizedError {
    pub message: String,
    pub stack: String,
    pub context: ErrorContext,
    pub hash: String,
}

pub fn validate(raw: &RawError) -> Result<()> {
    if raw.message.trim().is_empty() || raw.message.len() > 1024 {
        return Err(FaultReportError::Validation("Invalid message".to_string()).into());
    }
    if raw.context.url.len() > 2048 {
        return Err(FaultReportError::Validation("URL too long".to_string()).into());
    }
    Ok(())
}

pub fn normalize(raw: &RawError) -> Result<NormalizedError> {
    validate(raw)?;

    let message = raw.message.trim().to_string();
    let stack = raw.stack.clone().unwrap_or_default();

    let hash = compute_hash(&message, &stack, &raw.context.url);

    Ok(NormalizedError {
        message,
        stack,
        context: raw.context.clone(),
        hash,
    })
}

// IMMUTABLE from ARCHITECTURE.md 4.1
pub fn compute_hash(message: &str, stack: &str, url: &str) -> String {
    let message_trim = message.trim();

    let stack_frames: Vec<&str> = stack.lines().take(10).collect();
    let stack_clean = stack_frames.join("\n");

    let url_clean = url.split('?').next().unwrap_or(url).split('#').next().unwrap_or(url);

    let input = format!("{}\n{}\n{}", message_trim, stack_clean, url_clean);
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid() {
        let raw = RawError {
            message: "Error msg".to_string(),
            stack: Some("stack".to_string()),
            context: ErrorContext {
                url: "https://test.com".to_string(),
                browser: None,
                os: None,
                user_id: None,
                custom: serde_json::json!({}),
            },
        };
        assert!(validate(&raw).is_ok());
    }

    #[test]
    fn test_hash_deterministic() {
        let msg = "Cannot read property 'x' of undefined";
        let stk = "at foo.js:42\nat bar.js:100";
        let url = "https://example.com/page?foo=bar#hash";

        let hash1 = compute_hash(msg, stk, url);
        let hash2 = compute_hash(msg, stk, url);
        assert_eq!(hash1, hash2);

        // Ignores query/fragment
        let hash3 = compute_hash(msg, stk, "https://example.com/page?baz=qux");
        assert_eq!(hash1, hash3);
    }

    #[test]
    fn test_hash_first_10_frames() {
        let long_stack = "f1\nf2\nf3\nf4\nf5\nf6\nf7\nf8\nf9\nf10\nf11\nf12\nf13\nf14\nf15".to_string();
        let short_stack = "f1\nf2\nf3\nf4\nf5\nf6\nf7\nf8\nf9\nf10".to_string();
        let msg = "err";
        let url = "https://test.com";
        assert_eq!(
            compute_hash(msg, &long_stack, url),
            compute_hash(msg, &short_stack, url)
        );
    }
}

