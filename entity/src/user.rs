use sea_query::Iden;
use uuid::Uuid;

#[derive(Iden)]
pub enum User {
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
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub is_active: bool,
    pub email_verified_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
