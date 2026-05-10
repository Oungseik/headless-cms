use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::app::AppState;
use crate::app::error::{AppError, AppResult, ErrorResponse};

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[derive(Serialize, ToSchema)]
pub struct LogoutResponse {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/logout",
    operation_id = "logout",
    description = "Logout by revoking the refresh token",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "Logged out successfully", body = LogoutResponse),
        (status = 401, description = "Invalid refresh token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Auth",
)]
#[tracing::instrument]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LogoutRequest>,
) -> AppResult<Json<LogoutResponse>> {
    state
        .auth_service
        .logout(&body.refresh_token)
        .await
        .map_err(AppError::from)?;
    Ok(Json(LogoutResponse {
        message: "Logged out successfully".into(),
    }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::Json;
    use axum::extract::State;

    use crate::app::AppState;
    use crate::features::auth::logout::LogoutRequest;
    use crate::features::auth::service::AuthService;
    use crate::features::auth::service_mock::tests::MockAuthService;
    use crate::features::users::service_mock::tests::MockUserService;

    fn setup_app_state(auth: Arc<dyn AuthService>) -> Arc<AppState> {
        Arc::new(AppState {
            db: sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
            user_service: Arc::new(MockUserService::new()),
            auth_service: auth,
        })
    }

    /// Helper: register, verify, and login to obtain a refresh token.
    async fn get_refresh_token(mock: &Arc<MockAuthService>) -> String {
        mock.register("test@example.com", "password123", "customer")
            .await
            .unwrap();
        mock.verified_emails
            .lock()
            .expect("mutex poisoned")
            .insert("test@example.com".to_string());
        let login_res = mock.login("test@example.com", "password123").await.unwrap();
        login_res.refresh_token
    }

    #[tokio::test]
    async fn test_logout_success() {
        let mock = Arc::new(MockAuthService::new());
        let refresh_token = get_refresh_token(&mock).await;

        let state = setup_app_state(mock.clone());
        let result = super::handler(State(state), Json(LogoutRequest { refresh_token })).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_logout_idempotent() {
        let mock = Arc::new(MockAuthService::new());
        let refresh_token = get_refresh_token(&mock).await;

        let state = setup_app_state(mock.clone());

        let result1 = super::handler(
            State(state.clone()),
            Json(LogoutRequest {
                refresh_token: refresh_token.clone(),
            }),
        )
        .await;
        assert!(result1.is_ok());

        let result2 = super::handler(State(state), Json(LogoutRequest { refresh_token })).await;
        assert!(result2.is_ok());
    }
}
