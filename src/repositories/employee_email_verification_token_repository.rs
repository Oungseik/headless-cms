use chrono::{DateTime, Utc};
use sqlx::Sqlite;
use uuid::Uuid;

use crate::models::employee_email_verification_token::EmployeeEmailVerificationToken;

/// Insert a new email verification token.
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
        "INSERT INTO employee_email_verification_token (id, employee_id, token_hash, expires_at, created_at) VALUES (?, ?, ?, ?, ?)",
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

/// Find an email verification token by its hash.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the query fails.
pub async fn find_by_hash<'e, E: sqlx::Executor<'e, Database = Sqlite>>(
    executor: E,
    token_hash: &str,
) -> Result<Option<EmployeeEmailVerificationToken>, sqlx::Error> {
    sqlx::query_as::<_, EmployeeEmailVerificationToken>(
        "SELECT * FROM employee_email_verification_token WHERE token_hash = ?",
    )
    .bind(token_hash)
    .fetch_optional(executor)
    .await
}

/// Delete all email verification tokens for a specific employee.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the delete fails.
pub async fn delete_by_employee_id<'e, E: sqlx::Executor<'e, Database = Sqlite>>(
    executor: E,
    employee_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM employee_email_verification_token WHERE employee_id = ?")
        .bind(employee_id)
        .execute(executor)
        .await?;

    Ok(result.rows_affected())
}

/// Delete all email verification tokens.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the delete fails.
pub async fn delete_all<'e, E: sqlx::Executor<'e, Database = Sqlite>>(
    executor: E,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM employee_email_verification_token")
        .execute(executor)
        .await?;

    Ok(result.rows_affected())
}
