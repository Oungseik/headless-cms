use chrono::{DateTime, Utc};
use sqlx::Sqlite;
use uuid::Uuid;

use crate::models::employee_refresh_token::EmployeeRefreshToken;

/// Insert a new refresh token.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the insert fails.
pub async fn insert<'e, E: sqlx::Executor<'e, Database = Sqlite>>(
    executor: E,
    id: Uuid,
    employee_id: Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO employee_refresh_token (id, employee_id, token_hash, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(employee_id)
    .bind(token_hash)
    .bind(expires_at)
    .bind(created_at)
    .execute(executor)
    .await?;

    Ok(())
}

/// Find a refresh token by its hash.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the query fails.
#[expect(dead_code, reason = "will be used by token refresh handler")]
pub async fn find_by_hash<'e, E: sqlx::Executor<'e, Database = Sqlite>>(
    executor: E,
    token_hash: &str,
) -> Result<Option<EmployeeRefreshToken>, sqlx::Error> {
    sqlx::query_as::<_, EmployeeRefreshToken>(
        "SELECT * FROM employee_refresh_token WHERE token_hash = ?",
    )
    .bind(token_hash)
    .fetch_optional(executor)
    .await
}

/// Revoke a refresh token by setting `revoked_at`.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the update fails.
pub async fn revoke<'e, E: sqlx::Executor<'e, Database = Sqlite>>(
    executor: E,
    token_hash: &str,
    revoked_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE employee_refresh_token SET revoked_at = ? WHERE token_hash = ?")
        .bind(revoked_at)
        .bind(token_hash)
        .execute(executor)
        .await?;

    Ok(())
}
