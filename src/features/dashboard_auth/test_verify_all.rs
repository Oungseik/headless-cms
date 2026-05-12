use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    app::{
        AppState,
        error::{AppResult, ErrorResponse},
    },
    features::dashboard_auth::service::DashboardAuthService,
};

#[derive(Serialize, ToSchema)]
pub struct VerifyAllResponse {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/test/verify-all",
    operation_id = "dashboard_auth_test_verify_all",
    responses(
        (status = 200, description = "all employees verified", body = VerifyAllResponse),
        (status = 500, description = "internal server error", body = ErrorResponse),
    ),
    tag = "Dashboard Auth",
)]
#[tracing::instrument(skip(state))]
pub async fn handler(
    State(state): State<Arc<AppState>>,
) -> AppResult<(StatusCode, Json<VerifyAllResponse>)> {
    state.dashboard_auth_service.verify_all().await?;

    Ok((
        StatusCode::OK,
        Json(VerifyAllResponse {
            message: "All employees verified".to_string(),
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::dashboard_auth::service_impl::DashboardAuthServiceImpl;

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
    async fn verify_all_should_mark_employee_as_verified() {
        use crate::repositories::employee_repository;

        let service = setup_service().await;

        service
            .register("owner@example.com", "password1234")
            .await
            .expect("registration should succeed");

        let mut txn = service.pool.begin().await.unwrap();
        let employee = employee_repository::find_by_email(&mut *txn, "owner@example.com")
            .await
            .unwrap()
            .expect("employee should exist");
        assert!(
            employee.email_verified_at.is_none(),
            "email_verified_at should be None before verify_all"
        );
        txn.commit().await.unwrap();

        let result = service.verify_all().await;
        assert!(result.is_ok());

        let mut txn = service.pool.begin().await.unwrap();
        let employee = employee_repository::find_by_email(&mut *txn, "owner@example.com")
            .await
            .unwrap()
            .expect("employee should exist");
        assert!(
            employee.email_verified_at.is_some(),
            "email_verified_at should be Some after verify_all"
        );
        txn.commit().await.unwrap();
    }
}
