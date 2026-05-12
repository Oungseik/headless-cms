/// Hash a plaintext password using bcrypt.
///
/// # Errors
///
/// Returns [`bcrypt::BcryptError`] if hashing fails.
pub fn hash(password: &str, cost: u32) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, cost)
}

/// Verify a plaintext password against a bcrypt hash.
///
/// # Errors
///
/// Returns [`bcrypt::BcryptError`] if verification fails.
#[expect(dead_code, reason = "will be used by login handler")]
pub fn verify(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, hash)
}
