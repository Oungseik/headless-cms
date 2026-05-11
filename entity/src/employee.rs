use sea_query::Iden;
use uuid::Uuid;

/// Column identifiers for the `employee` table, used with [`SeaQuery`](sea_query).
#[derive(Iden)]
#[expect(missing_docs, reason = "variants mirror database column names")]
pub enum Employee {
    Table,
    Id,
    Email,
    PasswordHash,
    Role,
    IsActive,
    EmailVerifiedAt,
    CreatedAt,
    UpdatedAt,
}

/// A row from the `employee` table.
#[derive(sqlx::FromRow, Debug, Clone)]
#[expect(missing_docs, reason = "fields mirror database columns")]
pub struct EmployeeRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
    pub email_verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
