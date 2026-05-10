use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::app::AppState;
use crate::app::error::AppError;
use crate::auth::jwt::validate_access_token;

/// Extracts and validates JWT from the `Authorization: Bearer <token>` header.
///
/// Returns [`AppError::Unauthorized`] if the header is missing or the token is invalid.
#[derive(Debug)]
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
#[derive(Debug)]
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
            return Err(AppError::Forbidden("Admin access required".into()));
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::extract::FromRequestParts;
    use axum::http::Request;

    use crate::app::AppState;
    use crate::app::error::AppError;
    use crate::auth::jwt::generate_access_token;
    use crate::features::auth::service_mock::tests::MockAuthService;
    use crate::features::users::service_mock::tests::MockUserService;

    use super::{AdminUser, AuthUser, OptionalAuthUser};

    fn setup_state() -> Arc<AppState> {
        Arc::new(AppState {
            db: sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
            user_service: Arc::new(MockUserService::new()),
            auth_service: Arc::new(MockAuthService::new()),
        })
    }

    fn make_parts_with_token(token: &str) -> axum::http::request::Parts {
        let request = Request::builder()
            .header("Authorization", format!("Bearer {}", token))
            .body(())
            .unwrap();
        request.into_parts().0
    }

    fn make_parts_without_auth() -> axum::http::request::Parts {
        Request::builder().body(()).unwrap().into_parts().0
    }

    #[tokio::test]
    async fn test_auth_user_valid_token() {
        let state = setup_state();
        let token = generate_access_token(42, "admin").unwrap();
        let mut parts = make_parts_with_token(&token);
        let user = AuthUser::from_request_parts(&mut parts, &state)
            .await
            .unwrap();
        assert_eq!(user.user_id, 42);
        assert_eq!(user.role, "admin");
    }

    #[tokio::test]
    async fn test_auth_user_missing_header() {
        let state = setup_state();
        let mut parts = make_parts_without_auth();
        let result = AuthUser::from_request_parts(&mut parts, &state).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Unauthorized));
    }

    #[tokio::test]
    async fn test_auth_user_invalid_token() {
        let state = setup_state();
        let mut parts = make_parts_with_token("garbage.token.here");
        let result = AuthUser::from_request_parts(&mut parts, &state).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Unauthorized));
    }

    #[tokio::test]
    async fn test_admin_user_valid_admin() {
        let state = setup_state();
        let token = generate_access_token(1, "admin").unwrap();
        let mut parts = make_parts_with_token(&token);
        let admin = AdminUser::from_request_parts(&mut parts, &state)
            .await
            .unwrap();
        assert_eq!(admin.user_id, 1);
    }

    #[tokio::test]
    async fn test_admin_user_non_admin() {
        let state = setup_state();
        let token = generate_access_token(5, "customer").unwrap();
        let mut parts = make_parts_with_token(&token);
        let result = AdminUser::from_request_parts(&mut parts, &state).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_admin_user_missing_header() {
        let state = setup_state();
        let mut parts = make_parts_without_auth();
        let result = AdminUser::from_request_parts(&mut parts, &state).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::Unauthorized));
    }

    #[tokio::test]
    async fn test_optional_auth_user_no_token() {
        let state = setup_state();
        let mut parts = make_parts_without_auth();
        let opt_user = OptionalAuthUser::from_request_parts(&mut parts, &state)
            .await
            .unwrap();
        assert!(opt_user.0.is_none());
    }

    #[tokio::test]
    async fn test_optional_auth_user_valid_token() {
        let state = setup_state();
        let token = generate_access_token(99, "customer").unwrap();
        let mut parts = make_parts_with_token(&token);
        let opt_user = OptionalAuthUser::from_request_parts(&mut parts, &state)
            .await
            .unwrap();
        assert!(opt_user.0.is_some());
        let user = opt_user.0.unwrap();
        assert_eq!(user.user_id, 99);
        assert_eq!(user.role, "customer");
    }
}
