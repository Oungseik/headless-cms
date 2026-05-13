use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    app::{
        AppState,
        error::{AppResult, ErrorResponse},
    },
    features::dashboard_auth::service::DashboardAuthService,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyEmailQuery {
    /// Raw verification token from the email link.
    pub token: String,
}

#[derive(Serialize, ToSchema)]
pub struct VerifyEmailResponse {
    pub message: String,
}

#[utoipa::path(
    get,
    path = "/verify-email",
    operation_id = "dashboard_auth_verify_email",
    params(
        ("token" = String, Query, description = "Raw verification token"),
    ),
    responses(
        (status = 200, description = "email verified", body = VerifyEmailResponse),
        (status = 400, description = "invalid or expired token", body = ErrorResponse),
        (status = 404, description = "account not found", body = ErrorResponse),
        (status = 409, description = "email already verified", body = ErrorResponse),
    ),
    tag = "Dashboard Auth",
)]
#[tracing::instrument(skip(state))]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyEmailQuery>,
) -> AppResult<(StatusCode, Json<VerifyEmailResponse>)> {
    state
        .dashboard_auth_service
        .verify_email(&query.token)
        .await?;

    Ok((
        StatusCode::OK,
        Json(VerifyEmailResponse {
            message: "Email verified successfully.".to_string(),
        }),
    ))
}
