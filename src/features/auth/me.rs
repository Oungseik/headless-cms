use std::sync::Arc;

use axum::Json;
use axum::extract::State;

use crate::app::AppState;
use crate::app::error::{AppError, AppResult, ErrorResponse};
use crate::auth::extractor::AuthUser;
use crate::features::auth::service::MeResponse;

#[utoipa::path(
    get,
    path = "/me",
    operation_id = "me",
    description = "Get the currently authenticated user's profile",
    responses(
        (status = 200, description = "User profile", body = MeResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Authorization" = []),
        ("auth_token" = [])
    ),
    tag = "Auth",
)]
#[tracing::instrument]
pub async fn handler(
    AuthUser { user_id, .. }: AuthUser,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<MeResponse>> {
    let response = state
        .auth_service
        .get_me(user_id)
        .await
        .map_err(AppError::from)?;
    Ok(Json(response))
}
