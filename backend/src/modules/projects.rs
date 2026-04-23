use anyhow::Result;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::error::FaultReportError;
use crate::api_key::{generate_api_key, hash_api_key};

const MAX_PROJECTS_PER_USER: i64 = 10;

pub async fn create_project(pool: &PgPool, user_id: Uuid, name: &str) -> Result<(Uuid, String)> {
    // Enforce per-user project creation limit
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM projects WHERE created_by_user_id = $1"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    if count >= MAX_PROJECTS_PER_USER {
        return Err(FaultReportError::Validation(
            "Maximum project limit reached".to_string()
        ).into());
    }

    let plaintext_key = generate_api_key();
    let salt = Uuid::new_v4().as_simple().to_string(); // 32 chars without dashes, matches schema varchar(32)
    let key_hash = hash_api_key(&plaintext_key, &salt);

    let project_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO projects (created_by_user_id, name, api_key_hash, api_key_salt) VALUES ($1, $2, $3, $4) RETURNING id"
    )
    .bind(user_id)
    .bind(name)
    .bind(&key_hash)
    .bind(&salt)
    .fetch_one(pool)
    .await?;

    Ok((project_id, plaintext_key)) // Return project_id and api key
}

pub async fn verify_api_key(pool: &PgPool, key: &str) -> Result<(Uuid, Uuid)> {
    // Fetch all non-revoked projects with their salt
    let rows = sqlx::query(
        "SELECT id, created_by_user_id, api_key_hash, api_key_salt FROM projects WHERE revoked_at IS NULL"
    )
    .fetch_all(pool)
    .await?;

    // Hash the incoming key with each project's salt and compare
    for row in rows {
        let project_id: Uuid = row.get(0);
        let user_id: Uuid = row.get(1);
        let stored_hash: String = row.get(2);
        let salt: String = row.get(3);

        let computed_hash = hash_api_key(key, &salt);
        
        if computed_hash == stored_hash {
            return Ok((project_id, user_id));
        }
    }

    Err(FaultReportError::InvalidApiKey.into())
}

pub async fn rotate_api_key(pool: &PgPool, project_id: Uuid, user_id: Uuid) -> Result<String> {
    // Verify ownership
    let owner_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT created_by_user_id FROM projects WHERE id = $1"
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await?;

    let owner_id = owner_id.ok_or(FaultReportError::Unauthorized)?;
    if owner_id != user_id {
        return Err(FaultReportError::Unauthorized.into());
    }

    // Generate new
    let new_plaintext = generate_api_key();
    let salt = Uuid::new_v4().as_simple().to_string();
    let new_hash = hash_api_key(&new_plaintext, &salt);

    // Update (revoke old implicitly, new salt/hash)
    sqlx::query(
        "UPDATE projects SET api_key_hash = $1, api_key_salt = $2, revoked_at = NULL WHERE id = $3"
    )
    .bind(&new_hash)
    .bind(&salt)
    .bind(project_id)
    .execute(pool)
    .await?;

    Ok(new_plaintext)
}

/// Verify that the given user owns the specified project.
pub async fn require_project_owner(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> Result<(), FaultReportError> {
    let owner: Option<Uuid> = sqlx::query_scalar(
        "SELECT created_by_user_id FROM projects WHERE id = $1"
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| FaultReportError::Unauthorized)?;

    match owner {
        Some(id) if id == user_id => Ok(()),
        _ => Err(FaultReportError::Unauthorized),
    }
}
