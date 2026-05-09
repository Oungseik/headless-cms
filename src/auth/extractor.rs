use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::app::AppState;
use crate::app::error::AppError;
use crate::auth::jwt::validate_access_token;

/// Extracts and validates JWT from the `Authorization: Bearer <token>` header.
///
/// Returns [`AppError::Unauthorized`] if the header is missing or the token is invalid.
pub struct AuthUser {
    pub user_id: i32,
    pub role: String,
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .ok_or(AppError::Unauthorized)?
            .to_str()
            .map_err(|_| AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        let claims = validate_access_token(token).map_err(|_| AppError::Unauthorized)?;

        let user_id = claims
            .sub
            .parse::<i32>()
            .map_err(|_| AppError::Unauthorized)?;

        Ok(Self {
            user_id,
            role: claims.role,
        })
    }
}

/// Requires the authenticated user to have the "admin" role.
///
/// Returns [`AppError::Forbidden`] if the user is not an admin.
pub struct AdminUser {
    pub user_id: i32,
}

impl FromRequestParts<Arc<AppState>> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, state).await?;

        if auth_user.role != "admin" {
            return Err(AppError::Forbidden);
        }

        Ok(Self {
            user_id: auth_user.user_id,
        })
    }
}

/// Extracts the JWT if present but never fails — the inner [`Option`] is
/// [`None`] when no valid token is supplied and [`Some`] when authenticated.
pub struct OptionalAuthUser(pub Option<AuthUser>);

impl FromRequestParts<Arc<AppState>> for OptionalAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let inner = AuthUser::from_request_parts(parts, state).await.ok();
        Ok(Self(inner))
    }
}
