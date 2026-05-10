use sea_query::Iden;
use uuid::Uuid;

#[derive(Iden)]
pub enum EmployeeRefreshToken {
    Table,
    Id,
    EmployeeId,
    TokenHash,
    ExpiresAt,
    RevokedAt,
    CreatedAt,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct EmployeeRefreshTokenRow {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub token_hash: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
