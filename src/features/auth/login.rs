use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::app::AppState;
use crate::app::error::{AppError, AppResult, ErrorResponse};
use crate::features::auth::service::AuthResponse;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/login",
    operation_id = "login",
    description = "Login with username and password",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Auth",
)]
#[tracing::instrument]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let response = state
        .auth_service
        .login(body.username, body.password)
        .await
        .map_err(AppError::from)?;
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::Json;
    use axum::extract::State;

    use crate::app::AppState;
    use crate::features::auth::login::LoginRequest;
    use crate::features::auth::service::AuthService;
    use crate::features::auth::service_mock::tests::MockAuthService;
    use crate::features::users::service_mock::tests::MockUserService;

    fn setup_auth_service() -> Arc<dyn AuthService> {
        Arc::new(MockAuthService::new())
    }

    fn setup_app_state(auth: Arc<dyn AuthService>) -> Arc<AppState> {
        Arc::new(AppState {
            user_service: Arc::new(MockUserService::new()),
            auth_service: auth,
        })
    }

    #[tokio::test]
    async fn test_login_success() {
        let auth = setup_auth_service();
        auth.register(
            "testuser".into(),
            "test@example.com".into(),
            "password123".into(),
            "customer".into(),
        )
        .await
        .unwrap();

        let state = setup_app_state(auth);
        let result = super::handler(
            State(state),
            Json(LoginRequest {
                username: "testuser".into(),
                password: "password123".into(),
            }),
        )
        .await;

        let response = result.expect("login should succeed");
        assert_eq!(response.0.user.username, "testuser");
        assert!(!response.0.access_token.is_empty());
        assert!(!response.0.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let auth = setup_auth_service();
        auth.register(
            "testuser".into(),
            "test@example.com".into(),
            "password123".into(),
            "customer".into(),
        )
        .await
        .unwrap();

        let state = setup_app_state(auth);
        let result = super::handler(
            State(state),
            Json(LoginRequest {
                username: "testuser".into(),
                password: "wrongpassword".into(),
            }),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let auth = setup_auth_service();
        let state = setup_app_state(auth);

        let result = super::handler(
            State(state),
            Json(LoginRequest {
                username: "nonexistent".into(),
                password: "password123".into(),
            }),
        )
        .await;

        assert!(result.is_err());
    }
}
