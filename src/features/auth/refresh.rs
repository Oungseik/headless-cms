use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::app::AppState;
use crate::app::error::{AppError, AppResult, ErrorResponse};
use crate::features::auth::service::RefreshResponse;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[utoipa::path(
    post,
    path = "/refresh",
    operation_id = "refresh",
    description = "Refresh access token using a valid refresh token",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed", body = RefreshResponse),
        (status = 401, description = "Invalid or expired refresh token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Auth",
)]
#[tracing::instrument]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RefreshRequest>,
) -> AppResult<Json<RefreshResponse>> {
    let response = state
        .auth_service
        .refresh(&body.refresh_token)
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
    use crate::features::auth::refresh::RefreshRequest;
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
    async fn test_refresh_success() {
        let mock = Arc::new(MockAuthService::new());
        let refresh_token = get_refresh_token(&mock).await;

        let state = setup_app_state(mock.clone());
        let result = super::handler(State(state), Json(RefreshRequest { refresh_token })).await;

        let response = result.expect("refresh should succeed");
        assert!(!response.0.access_token.is_empty());
        assert!(!response.0.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_refresh_invalid_token() {
        let mock = Arc::new(MockAuthService::new());
        let state = setup_app_state(mock.clone());

        let result = super::handler(
            State(state),
            Json(RefreshRequest {
                refresh_token: "invalid_token".into(),
            }),
        )
        .await;

        assert!(result.is_err());
    }
}
