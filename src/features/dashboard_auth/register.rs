use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    app::{
        AppState,
        error::{AppResult, ErrorResponse},
    },
    features::dashboard_auth::service::DashboardAuthService,
};

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

impl std::fmt::Debug for RegisterRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisterRequest")
            .field("email", &self.email)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/register",
    operation_id = "dashboard_auth_register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "owner registered", body = RegisterResponse),
        (status = 400, description = "weak password", body = ErrorResponse),
        (status = 409, description = "owner already exists", body = ErrorResponse),
    ),
    tag = "Dashboard Auth",
)]
#[tracing::instrument(skip(state))]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<RegisterResponse>)> {
    state
        .dashboard_auth_service
        .register(&body.email, &body.password)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            message: "Please check your email to verify your account.".to_string(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::dashboard_auth::{
        service::DashboardAuthServiceError, service_impl::DashboardAuthServiceImpl,
    };

    async fn setup_service() -> DashboardAuthServiceImpl {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("failed to connect to in-memory sqlite");
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("failed to run migrations");
        DashboardAuthServiceImpl {
            pool,
            bcrypt_cost: 4,
            email_verification_token_ttl: 86400,
        }
    }

    #[tokio::test]
    async fn register_should_fail_when_password_too_short() {
        let service = setup_service().await;
        let result = service.register("owner@example.com", "short").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DashboardAuthServiceError::WeakPassword
        ));
    }

    #[tokio::test]
    async fn register_should_succeed_with_valid_input() {
        let service = setup_service().await;
        let result = service.register("owner@example.com", "password1234").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn register_should_fail_when_owner_already_exists() {
        let service = setup_service().await;
        service
            .register("owner@example.com", "password1234")
            .await
            .expect("first registration should succeed");

        let result = service.register("other@example.com", "password1234").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DashboardAuthServiceError::OwnerAlreadyExists
        ));
    }
}
