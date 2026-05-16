use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Invitation model — represents a pending employee invitation.
#[derive(Clone, Debug, sqlx::FromRow)]
#[expect(dead_code, reason = "used by repository queries")]
pub struct Invitation {
    /// Primary key — UUID v7 stored as text.
    pub id: Uuid,
    /// Invitee email address.
    pub email: String,
    /// Role to assign upon acceptance.
    pub role: String,
    /// SHA-256 hash of the raw invitation token.
    pub token_hash: String,
    /// FK to the employee who sent the invitation.
    pub invited_by: Uuid,
    /// Token expiry timestamp.
    pub expires_at: DateTime<Utc>,
    /// Timestamp when the invitation was accepted, if ever.
    pub accepted_at: Option<DateTime<Utc>>,
    /// Row creation timestamp.
    pub created_at: DateTime<Utc>,
}
