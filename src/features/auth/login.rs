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
    pub email: String,
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/login",
    operation_id = "login",
    description = "Login with email and password",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 403, description = "Email not verified", body = ErrorResponse),
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
        .login(&body.email, &body.password)
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

    fn setup_app_state(auth: Arc<dyn AuthService>) -> Arc<AppState> {
        Arc::new(AppState {
            db: sea_orm::DatabaseConnection::default(),
            user_service: Arc::new(MockUserService::new()),
            auth_service: auth,
        })
    }

    #[tokio::test]
    async fn test_login_success() {
        let mock = Arc::new(MockAuthService::new());
        mock.register("test@example.com", "password123", "customer")
            .await
            .unwrap();
        mock.verified_emails
            .lock()
            .expect("mutex poisoned")
            .insert("test@example.com".to_string());

        let state = setup_app_state(mock.clone());
        let result = super::handler(
            State(state),
            Json(LoginRequest {
                email: "test@example.com".into(),
                password: "password123".into(),
            }),
        )
        .await;

        let response = result.expect("login should succeed");
        assert_eq!(response.0.user.email, "test@example.com");
        assert_eq!(response.0.user.role, "customer");
        assert!(!response.0.access_token.is_empty());
        assert!(!response.0.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let mock = Arc::new(MockAuthService::new());
        mock.register("test@example.com", "password123", "customer")
            .await
            .unwrap();
        mock.verified_emails
            .lock()
            .expect("mutex poisoned")
            .insert("test@example.com".to_string());

        let state = setup_app_state(mock.clone());
        let result = super::handler(
            State(state),
            Json(LoginRequest {
                email: "test@example.com".into(),
                password: "wrongpassword".into(),
            }),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login_user_not_found() {
        let mock = Arc::new(MockAuthService::new());
        let state = setup_app_state(mock.clone());

        let result = super::handler(
            State(state),
            Json(LoginRequest {
                email: "nonexistent@example.com".into(),
                password: "password123".into(),
            }),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login_not_verified() {
        let mock = Arc::new(MockAuthService::new());
        mock.register("unverified@example.com", "password123", "customer")
            .await
            .unwrap();

        let state = setup_app_state(mock.clone());
        let result = super::handler(
            State(state),
            Json(LoginRequest {
                email: "unverified@example.com".into(),
                password: "password123".into(),
            }),
        )
        .await;

        assert!(result.is_err());
    }
}
