//! Authentication extractors for Axum route handlers.

use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, header::AUTHORIZATION, request::Parts},
};

use crate::{app::error::AppError, auth::jwt, config::get_config};

/// Claims extracted from a validated JWT access token.
///
/// Wraps [`jwt::Claims`] as a newtype to implement Axum's [`FromRequestParts`] trait.
/// Use as an Axum extractor to require authentication:
/// ```rust
/// async fn handler(claims: Claims) -> AppResult<Json<MeResponse>> {
///     // claims.sub is the employee ID (via Deref to jwt::Claims)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Claims(pub jwt::Claims);

impl std::ops::Deref for Claims {
    type Target = jwt::Claims;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = extract_bearer_token(&parts.headers)?;

        let config = get_config();
        let claims =
            jwt::decode(token, config.jwt_secret.as_bytes()).map_err(|_| AppError::Unauthorized)?;

        Ok(Self(claims))
    }
}

/// Extracts the Bearer token from the `Authorization` header.
fn extract_bearer_token(headers: &HeaderMap) -> Result<&str, AppError> {
    let header = headers
        .get(AUTHORIZATION)
        .ok_or(AppError::Unauthorized)?
        .to_str()
        .map_err(|_| AppError::Unauthorized)?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorized)?;

    if token.is_empty() {
        return Err(AppError::Unauthorized);
    }

    Ok(token)
}
