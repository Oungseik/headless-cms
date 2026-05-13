//! JWT encoding and decoding utilities.

use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Errors that can occur during JWT operations.
#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    /// The underlying JWT library returned an error.
    #[error("JWT operation failed: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    /// The current timestamp could not be represented as `usize`.
    #[error("timestamp overflow")]
    TimestampOverflow,
    /// The TTL value could not be represented as `usize`.
    #[error("TTL overflow")]
    TtlOverflow,
}

/// Claims embedded in a JWT access token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// The employee ID (`sub`ject).
    pub sub: Uuid,
    /// The employee's role.
    pub role: String,
    /// Expiration time as Unix timestamp.
    pub exp: usize,
}

/// Encodes a new JWT access token for the given employee.
///
/// # Errors
///
/// Returns [`JwtError`] if encoding fails or if the current timestamp or TTL
/// overflows `usize`.
pub fn encode(
    employee_id: Uuid,
    role: &str,
    secret: &[u8],
    ttl_seconds: u64,
) -> Result<String, JwtError> {
    let now = usize::try_from(Utc::now().timestamp()).map_err(|_| JwtError::TimestampOverflow)?;
    let ttl = usize::try_from(ttl_seconds).map_err(|_| JwtError::TtlOverflow)?;
    let exp = now + ttl;

    let claims = Claims {
        sub: employee_id,
        role: role.to_string(),
        exp,
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )?;

    Ok(token)
}

/// Decodes and validates a JWT access token, returning its claims.
///
/// # Errors
///
/// Returns [`JwtError`] if the token is invalid, expired, or cannot be decoded.
#[expect(dead_code, reason = "will be used by token refresh handler")]
pub fn decode(token: &str, secret: &[u8]) -> Result<Claims, JwtError> {
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}
