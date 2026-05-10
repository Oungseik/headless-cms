use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::get_config;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    /// User ID as UUID string
    pub sub: String,
    /// "admin" or "customer"
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    /// User ID as UUID string
    pub sub: String,
    /// Always "refresh"
    pub typ: String,
    pub exp: i64,
    pub iat: i64,
}

pub fn generate_access_token(
    user_id: Uuid,
    role: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let config = get_config();
    let now = Utc::now();
    let claims = TokenClaims {
        sub: user_id.to_string(),
        role: role.to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::seconds(config.access_token_ttl.cast_signed())).timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
}

pub fn generate_refresh_token(user_id: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let config = get_config();
    let now = Utc::now();
    let claims = RefreshTokenClaims {
        sub: user_id.to_string(),
        typ: "refresh".to_string(),
        iat: now.timestamp(),
        exp: (now + Duration::seconds(config.refresh_token_ttl.cast_signed())).timestamp(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate_access_token() {
        let user_id = Uuid::now_v7();
        let token = generate_access_token(user_id, "admin").unwrap();
        let claims = validate_access_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.role, "admin");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_generate_and_validate_refresh_token() {
        let user_id = Uuid::now_v7();
        let token = generate_refresh_token(user_id).unwrap();
        let claims = validate_refresh_token(&token).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.typ, "refresh");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn test_validate_access_token_invalid() {
        let result = validate_access_token("not.a.real.token");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_access_token_wrong_secret() {
        let user_id = Uuid::now_v7();
        let token = generate_access_token(user_id, "customer").unwrap();
        let wrong_key = jsonwebtoken::DecodingKey::from_secret(b"wrong-secret");
        let result = jsonwebtoken::decode::<TokenClaims>(
            &token,
            &wrong_key,
            &jsonwebtoken::Validation::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_refresh_token_with_access_token() {
        let user_id = Uuid::now_v7();
        let refresh_token = generate_refresh_token(user_id).unwrap();
        let result = validate_access_token(&refresh_token);
        assert!(
            result.is_err(),
            "refresh token should not be valid as access token"
        );
    }
}
