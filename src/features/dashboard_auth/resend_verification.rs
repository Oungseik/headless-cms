use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::app::AppState;
use crate::app::error::{AppError, AppResult, ErrorResponse};
use crate::features::dashboard_auth::service::DashboardResendVerificationResponse;

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResendVerificationRequest {
    pub email: String,
}

#[utoipa::path(
    post,
    path = "/resend-verification",
    operation_id = "dashboard_resend_verification",
    description = "Resend verification email. Returns generic message for security.",
    request_body = ResendVerificationRequest,
    responses(
        (status = 200, description = "Generic response", body = DashboardResendVerificationResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "DashboardAuth",
)]
#[tracing::instrument]
// TODO: Add rate limiting middleware (e.g. tower::limit::RateLimitLayer) to prevent email spam
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ResendVerificationRequest>,
) -> AppResult<Json<DashboardResendVerificationResponse>> {
    let response = state
        .dashboard_auth_service
        .resend_verification(&body.email)
        .await
        .map_err(AppError::from)?;
    Ok(Json(response))
}
