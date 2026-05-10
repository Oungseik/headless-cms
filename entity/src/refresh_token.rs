use sea_query::Iden;

#[derive(Iden)]
pub enum RefreshToken {
    Table,
    Id,
    UserId,
    TokenHash,
    ExpiresAt,
    RevokedAt,
    CreatedAt,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct RefreshTokenRow {
    pub id: i32,
    pub user_id: i32,
    pub token_hash: String,
    pub expires_at: chrono::NaiveDateTime,
    pub revoked_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}
