//! Logout endpoint for dashboard authentication.

use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    app::{AppState, error::AppResult},
    features::dashboard_auth::service::DashboardAuthService,
};

/// Request body for the logout endpoint.
#[derive(Deserialize, ToSchema)]
pub struct LogoutRequest {
    /// Hex-encoded refresh token to revoke.
    pub token: String,
}

impl std::fmt::Debug for LogoutRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogoutRequest")
            .field("token", &"[REDACTED]")
            .finish()
    }
}

/// Response body for a successful logout.
#[derive(Serialize, ToSchema)]
pub struct LogoutResponse {
    /// Human-readable confirmation message.
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/logout",
    operation_id = "dashboard_auth_logout",
    request_body = LogoutRequest,
    responses(
        (status = 200, description = "logout successful", body = LogoutResponse),
    ),
    tag = "Dashboard Auth",
)]
#[tracing::instrument(skip(state))]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LogoutRequest>,
) -> AppResult<(StatusCode, Json<LogoutResponse>)> {
    state.dashboard_auth_service.logout(&body.token).await?;

    Ok((
        StatusCode::OK,
        Json(LogoutResponse {
            message: "Logged out successfully.".to_string(),
        }),
    ))
}
