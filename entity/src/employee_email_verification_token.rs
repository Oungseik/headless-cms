use sea_query::Iden;
use uuid::Uuid;

#[derive(Iden)]
pub enum EmployeeEmailVerificationToken {
    Table,
    Id,
    EmployeeId,
    TokenHash,
    ExpiresAt,
    CreatedAt,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct EmployeeEmailVerificationTokenRow {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub token_hash: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
