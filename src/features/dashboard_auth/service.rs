#[derive(Debug, thiserror::Error)]
pub enum DashboardAuthServiceError {
    #[error("an owner has already been registered")]
    OwnerAlreadyExists,
    #[error("password must be at least 8 characters")]
    WeakPassword,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("email not verified")]
    EmailNotVerified,
    #[error("account is inactive")]
    AccountInactive,
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("password hashing failed")]
    PasswordHashing(#[from] bcrypt::BcryptError),
    #[error("JWT error")]
    Jwt(#[from] crate::auth::jwt::JwtError),
}

/// Result of a successful login.
#[derive(Debug)]
pub struct LoginResult {
    /// Signed JWT access token.
    pub access_token: String,
    /// Raw refresh token bytes (not yet encoded for transport).
    pub refresh_token: Vec<u8>,
    /// Access token TTL in seconds.
    pub expires_in: u64,
}

/// Service for dashboard authentication operations.
pub trait DashboardAuthService: Send + Sync + 'static {
    /// Registers the first owner account.
    ///
    /// Fails with [`DashboardAuthServiceError::OwnerAlreadyExists`] if an owner
    /// already exists, or [`DashboardAuthServiceError::WeakPassword`] if the
    /// password is shorter than 8 characters.
    async fn register(&self, email: &str, password: &str) -> Result<(), DashboardAuthServiceError>;

    /// Authenticates an employee and returns access + refresh tokens.
    ///
    /// Fails with [`DashboardAuthServiceError::InvalidCredentials`] if the email
    /// or password is wrong, [`DashboardAuthServiceError::EmailNotVerified`] if
    /// the email has not been verified, or [`DashboardAuthServiceError::AccountInactive`]
    /// if the account is deactivated.
    async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<LoginResult, DashboardAuthServiceError>;

    /// Marks all employees as email-verified and deletes all verification tokens.
    ///
    /// Intended for testing only.
    async fn verify_all(&self) -> Result<(), DashboardAuthServiceError>;
}
