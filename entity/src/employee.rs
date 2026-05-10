use sea_query::Iden;
use uuid::Uuid;

#[derive(Iden)]
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

#[derive(sqlx::FromRow, Debug, Clone)]
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
