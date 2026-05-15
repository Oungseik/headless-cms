//! Get current employee profile endpoint.

use std::sync::Arc;

use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    app::{
        AppState,
        error::{AppResult, ErrorResponse},
    },
    auth::extractor::Claims,
    repositories::employee_repository,
};

/// Response body containing the authenticated employee's profile.
#[derive(Debug, Serialize, ToSchema)]
pub struct MeResponse {
    /// Employee ID (UUID).
    pub id: Uuid,
    /// Email address.
    pub email: String,
    /// Role (e.g. "owner").
    pub role: String,
    /// Whether the account is active.
    pub is_active: bool,
    /// Timestamp when email was verified, if ever.
    pub email_verified_at: Option<DateTime<Utc>>,
    /// Account creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last update timestamp.
    pub updated_at: DateTime<Utc>,
}

#[utoipa::path(
    get,
    path = "/me",
    operation_id = "dashboard_auth_me",
    responses(
        (status = 200, description = "employee profile", body = MeResponse),
        (status = 401, description = "unauthorized", body = ErrorResponse),
        (status = 404, description = "employee not found", body = ErrorResponse),
    ),
    security(
        ("Authorization" = [])
    ),
    tag = "Dashboard Auth",
)]
#[tracing::instrument(skip(state))]
pub async fn handler(
    claims: Claims,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<MeResponse>> {
    let employee = employee_repository::find_by_id(&state.dashboard_auth_service.pool, claims.sub)
        .await
        .map_err(|e| {
            tracing::error!("failed to find employee: {e}");
            crate::app::error::AppError::InternalServerError
        })?
        .ok_or(crate::app::error::AppError::NotFound)?;

    Ok(Json(MeResponse {
        id: employee.id,
        email: employee.email,
        role: employee.role,
        is_active: employee.is_active,
        email_verified_at: employee.email_verified_at,
        created_at: employee.created_at,
        updated_at: employee.updated_at,
    }))
}
