use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Refresh token model for session management.
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct EmployeeRefreshToken {
    /// Primary key — UUID v7 stored as text.
    pub id: Uuid,
    /// FK to employee.
    pub employee_id: Uuid,
    /// SHA-256 hash of the raw token.
    pub token_hash: String,
    /// Token expiry timestamp.
    pub expires_at: DateTime<Utc>,
    /// Timestamp when the token was revoked, if ever.
    pub revoked_at: Option<DateTime<Utc>>,
    /// Row creation timestamp.
    pub created_at: DateTime<Utc>,
}
