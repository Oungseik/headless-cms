//! Login endpoint for dashboard authentication.

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

/// Request body for the login endpoint.
#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    /// Employee email address.
    pub email: String,
    /// Plaintext password.
    pub password: String,
}

// Redact the password field from debug/trace output to avoid leaking
// credentials into logs.
impl std::fmt::Debug for LoginRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginRequest")
            .field("email", &self.email)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

/// Response body containing access and refresh tokens.
#[derive(Serialize, ToSchema)]
pub struct TokenResponse {
    /// Signed JWT access token.
    pub access_token: String,
    /// Hex-encoded refresh token.
    pub refresh_token: String,
    /// Token type (always `"Bearer"`).
    pub token_type: String,
    /// Access token TTL in seconds.
    pub expires_in: u64,
}

#[utoipa::path(
    post,
    path = "/login",
    operation_id = "dashboard_auth_login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "login successful", body = TokenResponse),
        (status = 401, description = "invalid credentials", body = ErrorResponse),
        (status = 403, description = "email not verified or account inactive", body = ErrorResponse),
    ),
    tag = "Dashboard Auth",
)]
#[tracing::instrument(skip(state))]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> AppResult<(StatusCode, Json<TokenResponse>)> {
    let result = state
        .dashboard_auth_service
        .login(&body.email, &body.password)
        .await?;

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
