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
        .refresh(body.refresh_token)
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
    async fn test_refresh_success() {
        let auth = setup_auth_service();
        let reg = auth
            .register(
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
            Json(RefreshRequest {
                refresh_token: reg.refresh_token,
            }),
        )
        .await;

        let response = result.expect("refresh should succeed");
        assert!(!response.0.access_token.is_empty());
        assert!(!response.0.refresh_token.is_empty());
    }

    #[tokio::test]
    async fn test_refresh_invalid_token() {
        let auth = setup_auth_service();
        let state = setup_app_state(auth);

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
