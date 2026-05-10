use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::app::AppState;
use crate::app::error::{AppError, AppResult, ErrorResponse};
use crate::features::dashboard_auth::service::DashboardRegisterResponse;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/register",
    operation_id = "dashboard_register",
    description = "Register a new owner account. Only one owner allowed.",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Registration successful", body = DashboardRegisterResponse),
        (status = 400, description = "Invalid request (e.g. weak password)", body = ErrorResponse),
        (status = 409, description = "Owner already exists or email taken", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "DashboardAuth",
)]
#[tracing::instrument]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<DashboardRegisterResponse>)> {
    let response = state
        .dashboard_auth_service
        .register(&body.email, &body.password)
        .await
        .map_err(AppError::from)?;
    Ok((StatusCode::CREATED, Json(response)))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::Json;
    use axum::extract::State;
    use axum::http::StatusCode;

    use crate::app::AppState;
    use crate::features::dashboard_auth::register::RegisterRequest;
    use crate::features::dashboard_auth::service::DashboardAuthService;
    use crate::features::dashboard_auth::service_mock::MockDashboardAuthService;

    fn setup_auth_service() -> Arc<dyn DashboardAuthService> {
        Arc::new(MockDashboardAuthService::new())
    }

    fn setup_app_state(auth: Arc<dyn DashboardAuthService>) -> Arc<AppState> {
        Arc::new(AppState {
            db: sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
            dashboard_auth_service: auth,
        })
    }

    #[tokio::test]
    async fn test_register_success() {
        let auth = setup_auth_service();
        let state = setup_app_state(auth);

        let result = super::handler(
            State(state),
            Json(RegisterRequest {
                email: "new@example.com".into(),
                password: "password123".into(),
            }),
        )
        .await;

        let response = result.expect("register should succeed");
        assert_eq!(response.0, StatusCode::CREATED);
        assert!(!response.1.0.message.is_empty());
    }

    #[tokio::test]
    async fn test_register_duplicate_owner() {
        let auth = setup_auth_service();
        auth.register("a@b.com", "pass").await.unwrap();

        let state = setup_app_state(auth);
        let result = super::handler(
            State(state),
            Json(RegisterRequest {
                email: "c@d.com".into(),
                password: "pass".into(),
            }),
        )
        .await;

        assert!(result.is_err());
    }
}
