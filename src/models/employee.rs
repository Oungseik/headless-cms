use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Employee account model.
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Employee {
    /// Primary key — UUID v7 stored as text.
    pub id: Uuid,
    /// Unique email address.
    pub email: String,
    /// Bcrypt password hash.
    pub password_hash: String,
    /// Role (e.g. `owner`).
    pub role: String,
    /// Whether the account is active.
    pub is_active: bool,
    /// Timestamp when email was verified, if ever.
    pub email_verified_at: Option<DateTime<Utc>>,
    /// Row creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Row last-updated timestamp.
    pub updated_at: DateTime<Utc>,
}
