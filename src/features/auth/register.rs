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
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[utoipa::path(
    post,
    path = "",
    description = "Register a new customer account",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Registration successful", body = AuthResponse),
        (status = 409, description = "Username already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Auth",
)]
#[tracing::instrument]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    let response = state
        .auth_service
        .register(body.username, body.email, body.password, "customer".into())
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
    use crate::features::auth::register::RegisterRequest;
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
    async fn test_register_success() {
        let auth = setup_auth_service();
        let state = setup_app_state(auth);

        let result = super::handler(
            State(state),
            Json(RegisterRequest {
                username: "newuser".into(),
                email: "new@example.com".into(),
                password: "password123".into(),
            }),
        )
        .await;

        let response = result.expect("register should succeed");
        assert_eq!(response.0.user.username, "newuser");
        assert_eq!(response.0.user.role, "customer");
        assert!(!response.0.access_token.is_empty());
    }

    #[tokio::test]
    async fn test_register_duplicate_username() {
        let auth = setup_auth_service();
        auth.register(
            "existing".into(),
            "a@b.com".into(),
            "pass".into(),
            "customer".into(),
        )
        .await
        .unwrap();

        let state = setup_app_state(auth);
        let result = super::handler(
            State(state),
            Json(RegisterRequest {
                username: "existing".into(),
                email: "other@b.com".into(),
                password: "pass".into(),
            }),
        )
        .await;

        assert!(result.is_err());
    }
}
