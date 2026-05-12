#[derive(Debug, thiserror::Error)]
pub enum DashboardAuthServiceError {
    #[error("an owner has already been registered")]
    OwnerAlreadyExists,
    #[error("password must be at least 8 characters")]
    WeakPassword,
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("password hashing failed")]
    PasswordHashing(#[from] bcrypt::BcryptError),
}

/// Service for dashboard authentication operations.
pub trait DashboardAuthService: Send + Sync + 'static {
    /// Registers the first owner account.
    ///
    /// Fails with [`DashboardAuthServiceError::OwnerAlreadyExists`] if an owner
    /// already exists, or [`DashboardAuthServiceError::WeakPassword`] if the
    /// password is shorter than 8 characters.
    async fn register(&self, email: &str, password: &str) -> Result<(), DashboardAuthServiceError>;

    /// Marks all employees as email-verified and deletes all verification tokens.
    ///
    /// Intended for testing only.
    async fn verify_all(&self) -> Result<(), DashboardAuthServiceError>;
}
