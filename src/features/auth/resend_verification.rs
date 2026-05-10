use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::app::AppState;
use crate::app::error::{AppError, AppResult, ErrorResponse};
use crate::features::auth::service::ResendVerificationResponse;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResendVerificationRequest {
    pub email: String,
}

#[utoipa::path(
    post,
    path = "/resend-verification",
    operation_id = "resend_verification",
    description = "Resend the email verification link",
    request_body = ResendVerificationRequest,
    responses(
        (status = 200, description = "Verification email resent", body = ResendVerificationResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Auth",
)]
#[tracing::instrument]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ResendVerificationRequest>,
) -> AppResult<Json<ResendVerificationResponse>> {
    let response = state
        .auth_service
        .resend_verification(&body.email)
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
    use crate::features::auth::resend_verification::ResendVerificationRequest;
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

    #[tokio::test]
    async fn test_resend_verification_success() {
        let mock = Arc::new(MockAuthService::new());
        mock.register("test@example.com", "password123", "customer")
            .await
            .unwrap();

        let state = setup_app_state(mock.clone());
        let result = super::handler(
            State(state),
            Json(ResendVerificationRequest {
                email: "test@example.com".into(),
            }),
        )
        .await;

        let response = result.expect("resend verification should succeed");
        assert!(!response.0.message.is_empty());
    }
}
