//! Token refresh endpoint for dashboard authentication.

use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    app::{
        AppState,
        error::{AppResult, ErrorResponse},
    },
    features::dashboard_auth::{login::TokenResponse, service::DashboardAuthService},
};

/// Request body for the token refresh endpoint.
#[derive(Deserialize, ToSchema)]
pub struct RefreshRequest {
    /// Hex-encoded refresh token.
    pub token: String,
}

impl std::fmt::Debug for RefreshRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefreshRequest")
            .field("token", &"[REDACTED]")
            .finish()
    }
}

#[utoipa::path(
    post,
    path = "/refresh",
    operation_id = "dashboard_auth_refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "token refreshed", body = TokenResponse),
        (status = 401, description = "invalid or expired refresh token", body = ErrorResponse),
        (status = 403, description = "account inactive", body = ErrorResponse),
    ),
    tag = "Dashboard Auth",
)]
#[tracing::instrument(skip(state))]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RefreshRequest>,
) -> AppResult<(StatusCode, Json<TokenResponse>)> {
    let result = state.dashboard_auth_service.refresh(&body.token).await?;

    Ok((
        StatusCode::OK,
        Json(TokenResponse {
            access_token: result.access_token,
            refresh_token: hex::encode(result.refresh_token),
            token_type: "Bearer".to_string(),
            expires_in: result.expires_in,
        }),
    ))
}
