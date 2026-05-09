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
    path = "",
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
        .logout(body.refresh_token)
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
    async fn test_logout_success() {
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
            Json(LogoutRequest {
                refresh_token: reg.refresh_token,
            }),
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_logout_idempotent() {
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

        let state = setup_app_state(auth.clone());

        let result1 = super::handler(
            State(state.clone()),
            Json(LogoutRequest {
                refresh_token: reg.refresh_token.clone(),
            }),
        )
        .await;
        assert!(result1.is_ok());

        let result2 = super::handler(
            State(state),
            Json(LogoutRequest {
                refresh_token: reg.refresh_token,
            }),
        )
        .await;
        assert!(result2.is_ok());
    }
}
