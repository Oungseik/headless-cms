use chrono::{DateTime, Utc};
use sqlx::Sqlite;
use uuid::Uuid;

use crate::models::invitation::Invitation;

/// Data needed to insert a new invitation.
pub struct NewInvitation<'a> {
    pub id: Uuid,
    pub email: &'a str,
    pub role: &'a str,
    pub token_hash: &'a str,
    pub invited_by: Uuid,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Insert a new invitation.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the insert fails.
pub async fn insert<'e, E: sqlx::Executor<'e, Database = Sqlite>>(
    executor: E,
    invitation: &NewInvitation<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO invitation (id, email, role, token_hash, invited_by, expires_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(invitation.id)
    .bind(invitation.email)
    .bind(invitation.role)
    .bind(invitation.token_hash)
    .bind(invitation.invited_by)
    .bind(invitation.expires_at)
    .bind(invitation.created_at)
    .execute(executor)
    .await?;

    Ok(())
}

/// Find a pending (not yet accepted) invitation by email.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the query fails.
pub async fn find_pending_by_email<'e, E: sqlx::Executor<'e, Database = Sqlite>>(
    executor: E,
    email: &str,
) -> Result<Option<Invitation>, sqlx::Error> {
    sqlx::query_as::<_, Invitation>(
        "SELECT * FROM invitation WHERE email = ? AND accepted_at IS NULL",
    )
    .bind(email)
    .fetch_optional(executor)
    .await
}

/// Delete a pending invitation by email.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the delete fails.
pub async fn delete_pending_by_email<'e, E: sqlx::Executor<'e, Database = Sqlite>>(
    executor: E,
    email: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM invitation WHERE email = ? AND accepted_at IS NULL")
        .bind(email)
        .execute(executor)
        .await?;

    Ok(result.rows_affected())
}
