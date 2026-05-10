use sea_query::Iden;

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
    pub id: i32,
    pub user_id: i32,
    pub token_hash: String,
    pub expires_at: chrono::DateTime<chrono::FixedOffset>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}
