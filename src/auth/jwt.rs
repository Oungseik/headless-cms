use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::config::get_config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    /// User ID as string
    pub sub: String,
    /// "admin" or "customer"
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    /// User ID as string
    pub sub: String,
    /// Always "refresh"
    pub typ: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn generate_access_token(
    user_id: i32,
    role: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let config = get_config();
    let now = Utc::now();
    let claims = TokenClaims {
        sub: user_id.to_string(),
        role: role.to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::seconds(config.access_token_ttl as i64)).timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
}

pub fn generate_refresh_token(user_id: i32) -> Result<String, jsonwebtoken::errors::Error> {
    let config = get_config();
    let now = Utc::now();
    let claims = RefreshTokenClaims {
        sub: user_id.to_string(),
        typ: "refresh".to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::seconds(config.refresh_token_ttl as i64)).timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
}

pub fn validate_access_token(token: &str) -> Result<TokenClaims, jsonwebtoken::errors::Error> {
    let config = get_config();
    let token_data = decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

pub fn validate_refresh_token(
    token: &str,
) -> Result<RefreshTokenClaims, jsonwebtoken::errors::Error> {
    let config = get_config();
    let token_data = decode::<RefreshTokenClaims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}
