use chrono::{DateTime, Utc};
use sqlx::Sqlite;
use uuid::Uuid;

use crate::models::employee::Employee;

/// Count all employees.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the query fails.
pub async fn count_all<'e, E: sqlx::Executor<'e, Database = Sqlite>>(
    executor: E,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM employee")
        .fetch_one(executor)
        .await
}

/// Find an employee by email.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the query fails.
pub async fn find_by_email<'e, E: sqlx::Executor<'e, Database = Sqlite>>(
    executor: E,
    email: &str,
) -> Result<Option<Employee>, sqlx::Error> {
    sqlx::query_as::<_, Employee>("SELECT * FROM employee WHERE email = ?")
        .bind(email)
        .fetch_optional(executor)
        .await
}

/// Insert a new employee.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the insert fails.
pub async fn insert<'e, E: sqlx::Executor<'e, Database = Sqlite>>(
    executor: E,
    id: Uuid,
    email: &str,
    password_hash: &str,
    role: &str,
    is_active: bool,
    email_verified_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO employee (id, email, password_hash, role, is_active, email_verified_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .bind(role)
    .bind(is_active)
    .bind(email_verified_at)
    .bind(created_at)
    .bind(updated_at)
    .execute(executor)
    .await?;

    Ok(())
}
