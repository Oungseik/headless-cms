use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::app::AppState;
use crate::app::error::{AppError, AppResult, ErrorResponse};
use crate::features::auth::service::VerifyEmailResponse;

#[derive(Debug, Deserialize, IntoParams)]
pub struct VerifyEmailQuery {
    pub token: String,
}

#[utoipa::path(
    get,
    path = "/verify-email",
    operation_id = "verify_email",
    description = "Verify email address using the token sent via email",
    params(VerifyEmailQuery),
    responses(
        (status = 200, description = "Email verified successfully", body = VerifyEmailResponse),
        (status = 404, description = "Invalid verification token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Auth",
)]
#[tracing::instrument]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<VerifyEmailQuery>,
) -> AppResult<Json<VerifyEmailResponse>> {
    let response = state
        .auth_service
        .verify_email(&params.token)
        .await
        .map_err(AppError::from)?;
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::extract::{Query, State};

    use crate::app::AppState;
    use crate::features::auth::service::AuthService;
    use crate::features::auth::service_mock::tests::MockAuthService;
    use crate::features::auth::verify_email::VerifyEmailQuery;
    use crate::features::users::service_mock::tests::MockUserService;

    fn setup_app_state(auth: Arc<dyn AuthService>) -> Arc<AppState> {
        Arc::new(AppState {
            db: sea_orm::DatabaseConnection::default(),
            user_service: Arc::new(MockUserService::new()),
            auth_service: auth,
        })
    }

    #[tokio::test]
    async fn test_verify_email_success() {
        let mock = Arc::new(MockAuthService::new());
        mock.register("test@example.com", "password123", "customer")
            .await
            .unwrap();

        // Insert a verification token into the mock
        mock.verification_tokens
            .lock()
            .expect("mutex poisoned")
            .insert("valid-token".to_string(), "test@example.com".to_string());

        let state = setup_app_state(mock.clone());
        let result = super::handler(
            State(state),
            Query(VerifyEmailQuery {
                token: "valid-token".to_string(),
            }),
        )
        .await;

        let response = result.expect("verify email should succeed");
        assert!(!response.0.message.is_empty());

        // Confirm email is now verified
        assert!(
            mock.verified_emails
                .lock()
                .expect("mutex poisoned")
                .contains("test@example.com")
        );
    }

    #[tokio::test]
    async fn test_verify_email_invalid_token() {
        let mock = Arc::new(MockAuthService::new());
        let state = setup_app_state(mock.clone());

        let result = super::handler(
            State(state),
            Query(VerifyEmailQuery {
                token: "invalid-token".to_string(),
            }),
        )
        .await;

        assert!(result.is_err());
    }
}
