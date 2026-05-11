use sea_query::Iden;
use uuid::Uuid;

/// Column identifiers for the `employee_email_verification_token` table.
#[derive(Iden)]
#[expect(missing_docs, reason = "variants mirror database column names")]
pub enum EmployeeEmailVerificationToken {
    Table,
    Id,
    EmployeeId,
    TokenHash,
    ExpiresAt,
    CreatedAt,
}

/// A row from the `employee_email_verification_token` table.
#[derive(sqlx::FromRow, Debug, Clone)]
#[expect(missing_docs, reason = "fields mirror database columns")]
pub struct EmployeeEmailVerificationTokenRow {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub token_hash: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
