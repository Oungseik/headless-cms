use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Email verification token model.
#[derive(Clone, Debug, sqlx::FromRow)]
#[expect(dead_code, reason = "used by repository queries")]
pub struct EmployeeEmailVerificationToken {
    /// Primary key — UUID v7 stored as text.
    pub id: Uuid,
    /// FK to employee.
    pub employee_id: Uuid,
    /// SHA-256 hash of the raw token.
    pub token_hash: String,
    /// Token expiry timestamp.
    pub expires_at: DateTime<Utc>,
    /// Row creation timestamp.
    pub created_at: DateTime<Utc>,
}
