use sea_query::Iden;
use uuid::Uuid;

#[derive(Iden)]
pub enum EmailVerificationToken {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    CreatedAt,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct EmailVerificationTokenRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: chrono::DateTime<chrono::FixedOffset>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}
