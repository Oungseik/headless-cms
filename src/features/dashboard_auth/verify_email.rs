use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::app::AppState;
use crate::app::error::{AppError, AppResult, ErrorResponse};
use crate::features::dashboard_auth::service::DashboardVerifyEmailResponse;

#[derive(Debug, Deserialize, IntoParams)]
pub struct VerifyEmailQuery {
    pub token: String,
}

#[utoipa::path(
    get,
    path = "/verify-email",
    operation_id = "dashboard_verify_email",
    description = "Verify owner email via token from verification email.",
    params(VerifyEmailQuery),
    responses(
        (status = 200, description = "Email verified", body = DashboardVerifyEmailResponse),
        (status = 404, description = "Invalid or expired token", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "DashboardAuth",
)]
#[tracing::instrument]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<VerifyEmailQuery>,
) -> AppResult<Json<DashboardVerifyEmailResponse>> {
    let response = state
        .dashboard_auth_service
        .verify_email(&params.token)
        .await
        .map_err(AppError::from)?;
    Ok(Json(response))
}
