#[derive(Debug, thiserror::Error)]
pub enum DashboardInvitationServiceError {
    #[error("cannot invite with owner role")]
    OwnerRoleForbidden,
    #[error("email is already registered as an employee")]
    EmailAlreadyEmployee,
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

/// Service for dashboard invitation operations.
pub trait DashboardInvitationService: Send + Sync + 'static {
    /// Creates an invitation for the given email with the specified role.
    ///
    /// If a pending invitation already exists for the email it is replaced.
    ///
    /// Returns the hex-encoded invitation token on success.
    ///
    /// Fails with [`DashboardInvitationServiceError::OwnerRoleForbidden`] if
    /// the role is `"owner"`, or [`DashboardInvitationServiceError::EmailAlreadyEmployee`]
    /// if the email is already registered as an employee.
    async fn invite(
        &self,
        email: &str,
        role: &str,
        invited_by: uuid::Uuid,
    ) -> Result<String, DashboardInvitationServiceError>;
}
