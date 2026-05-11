use sea_query::Iden;
use uuid::Uuid;

/// Column identifiers for the `employee_refresh_token` table.
#[derive(Iden)]
#[expect(missing_docs, reason = "variants mirror database column names")]
pub enum EmployeeRefreshToken {
    Table,
    Id,
    EmployeeId,
    TokenHash,
    ExpiresAt,
    RevokedAt,
    CreatedAt,
}

/// A row from the `employee_refresh_token` table.
#[derive(sqlx::FromRow, Debug, Clone)]
#[expect(missing_docs, reason = "fields mirror database columns")]
pub struct EmployeeRefreshTokenRow {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub token_hash: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
