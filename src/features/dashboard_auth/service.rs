use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum DashboardAuthServiceError {
    #[error("an owner has already been registered")]
    OwnerAlreadyExists,
    #[error("password must be at least 8 characters")]
    WeakPassword,
    #[error("database error")]
    Database(#[from] sea_orm::DbErr),
    #[error("password hashing failed")]
    PasswordHashing(#[from] bcrypt::BcryptError),
}

/// Service for dashboard authentication operations.
#[async_trait]
pub trait DashboardAuthService: Send + Sync + 'static {
    /// Registers the first owner account.
    ///
    /// Fails with [`DashboardAuthServiceError::OwnerAlreadyExists`] if an owner
    /// already exists, or [`DashboardAuthServiceError::WeakPassword`] if the
    /// password is shorter than 8 characters.
    async fn register<'a>(
        &'a self,
        email: &'a str,
        password: &'a str,
    ) -> Result<(), DashboardAuthServiceError>;
}
