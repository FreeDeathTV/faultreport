use actix_web::{web, HttpRequest, HttpResponse, Responder};
use sqlx::PgPool;
use crate::error::FaultReportError;
use crate::auth::cache::ApiKeyCache;
use crate::modules::{projects, error_capture, storage, alert};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateProjectResponse {
    pub project_id: Uuid,
    pub api_key: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn health(pool: web::Data<PgPool>) -> impl Responder {
    let db_ok = pool.acquire().await.is_ok();

    // Verify required tables (errors, ledger, projects, users) to infer migrations ran
    let required_tables: Vec<&str> = vec!["errors", "ledger", "projects", "users"];
    let tables_ok = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM information_schema.tables
         WHERE table_schema = 'public'
         AND table_name = ANY($1)"
    )
    .bind(&required_tables)
    .fetch_one(pool.as_ref())
    .await
    .map(|count| count >= 4)
    .unwrap_or(false);

    let mut status = if db_ok && tables_ok {
        HttpResponse::Ok()
    } else {
        HttpResponse::ServiceUnavailable()
    };

    status
        .append_header(("Content-Security-Policy", "default-src 'self'"))
        .append_header(("X-Content-Type-Options", "nosniff"))
        .append_header(("X-Frame-Options", "DENY"))
        .json(json!({
            "status": if db_ok && tables_ok { "healthy" } else { "unhealthy" },
            "database": db_ok,
            "migrations_applied": tables_ok
        }))
}

// Lightweight health endpoint that does not require DB connectivity.
pub async fn simple_health() -> impl Responder {
    HttpResponse::Ok()
        .append_header(("Content-Security-Policy", "default-src 'self'"))
        .append_header(("X-Content-Type-Options", "nosniff"))
        .append_header(("X-Frame-Options", "DENY"))
        .json(json!({
            "status": "ok",
            "server": "running"
        }))
}

pub async fn submit_error(
    pool: web::Data<PgPool>,
    cache: web::Data<ApiKeyCache>,
    req: HttpRequest,
    raw_error: web::Json<error_capture::RawError>,
) -> Result<HttpResponse, FaultReportError> {
    // Extract API key from Authorization: Bearer frp_...
    let key = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(FaultReportError::InvalidApiKey)?;

    // Verify key → project_id, user_id (uses cache)
    let (project_id, _user_id) = cache.validate_key(&pool, key).await?;

    // Rate limit
    if !storage::check_rate_limit(&pool, project_id).await? {
        return Err(FaultReportError::RateLimitExceeded);
    }

    // Capture + hash
    let normalized = error_capture::normalize(&raw_error).map_err(|e| FaultReportError::Validation(e.to_string()))?;
    
    // Persist
    let (error_id, was_duplicate, count) = storage::persist(&pool, project_id, normalized.clone()).await?;

    // Clone hash for spawn (move)
    let error_hash = normalized.hash.clone();

    // Check spike → alert (fire-forget)
    if alert::check_spike(&pool, project_id, &error_hash).await? && alert::should_alert(&pool, project_id, &error_hash).await? {
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            let _ = alert::record_alert(&pool_clone, project_id, &error_hash).await;
            let _ = alert::post_slack(&pool_clone, project_id, &error_hash).await;
        });
    }

Ok(HttpResponse::Created()
        .append_header(("Content-Security-Policy", "default-src 'self'"))
        .append_header(("X-Content-Type-Options", "nosniff"))
        .append_header(("X-Frame-Options", "DENY"))
        .json(json!({
            "id": error_id,
            "hash": normalized.hash,
            "was_duplicate": was_duplicate,
            "count": count
        })))
}

pub async fn list_errors(
    pool: web::Data<PgPool>,
    cache: web::Data<ApiKeyCache>,
    req: HttpRequest,
    path: web::Path<(String,)>,
) -> Result<HttpResponse, FaultReportError> {
    let project_id = Uuid::parse_str(&path.0)
        .map_err(|_| FaultReportError::InvalidApiKey)?;

    // Require API key for listing errors
    let key = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(FaultReportError::InvalidApiKey)?;

    let (verified_project, _user) = cache.validate_key(&pool, key).await?;
    if verified_project != project_id {
        return Err(FaultReportError::Unauthorized);
    }

    let errors = storage::list_errors(&pool, project_id, 20).await?;
    Ok(HttpResponse::Ok()
        .append_header(("Content-Security-Policy", "default-src 'self'"))
        .append_header(("X-Content-Type-Options", "nosniff"))
        .append_header(("X-Frame-Options", "DENY"))
        .json(json!({
            "errors": errors,
            "total": errors.len(),
            "page": 1,
            "per_page": 20
        })))
}

pub async fn create_project(
    pool: web::Data<PgPool>,
    cache: web::Data<ApiKeyCache>,
    req: HttpRequest,
    req_body: web::Json<CreateProjectRequest>,
) -> Result<HttpResponse, FaultReportError> {
    // Authenticate user (Firebase or mock). Use middleware helper which
    // supports local `mock:<uuid>` tokens for tests.
    let user_id = crate::middleware::auth::require_firebase_auth(&req).await?;

    let (project_id, api_key) = projects::create_project(&pool, user_id, &req_body.name).await?;

    Ok(HttpResponse::Created()
        .append_header(("Content-Security-Policy", "default-src 'self'"))
        .append_header(("X-Content-Type-Options", "nosniff"))
        .append_header(("X-Frame-Options", "DENY"))
        .json(CreateProjectResponse {
            project_id,
            api_key,
            created_at: chrono::Utc::now(),
        }))
}

