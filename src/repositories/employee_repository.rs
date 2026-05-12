use chrono::{DateTime, Utc};
use sqlx::Sqlite;
use uuid::Uuid;

use crate::models::employee::Employee;

/// Parameters for creating a new employee.
pub struct CreateEmployee<'a> {
    pub id: Uuid,
    pub email: &'a str,
    pub password_hash: &'a str,
    pub role: &'a str,
    pub is_active: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

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
#[expect(dead_code, reason = "will be used by login handler")]
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
    employee: CreateEmployee<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO employee (id, email, password_hash, role, is_active, email_verified_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(employee.id)
    .bind(employee.email)
    .bind(employee.password_hash)
    .bind(employee.role)
    .bind(employee.is_active)
    .bind(employee.email_verified_at)
    .bind(employee.created_at)
    .bind(employee.updated_at)
    .execute(executor)
    .await?;

    Ok(())
}
