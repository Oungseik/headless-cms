use std::sync::Arc;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    app::{
        AppState,
        error::{AppError, AppResult, ErrorResponse},
    },
    auth::extractor::Claims,
    email::EmailSender,
    features::dashboard_invitations::service::DashboardInvitationService,
    repositories::employee_repository,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct InviteRequest {
    pub email: String,
    pub role: String,
}

#[derive(Serialize, ToSchema)]
pub struct InviteResponse {
    pub message: String,
    pub email: String,
    pub role: String,
}

#[utoipa::path(
    post,
    path = "",
    operation_id = "dashboard_invitations_create",
    request_body = InviteRequest,
    responses(
        (status = 200, description = "invitation sent", body = InviteResponse),
        (status = 400, description = "owner role forbidden", body = ErrorResponse),
        (status = 401, description = "unauthorized", body = ErrorResponse),
        (status = 403, description = "forbidden (non-owner)", body = ErrorResponse),
        (status = 409, description = "email already registered", body = ErrorResponse),
    ),
    security(
        ("Authorization" = [])
    ),
    tag = "Dashboard Invitations",
)]
#[tracing::instrument(skip(state))]
pub async fn handler(
    claims: Claims,
    State(state): State<Arc<AppState>>,
    Json(body): Json<InviteRequest>,
) -> AppResult<Json<InviteResponse>> {
    if claims.role != "owner" {
        return Err(AppError::Forbidden);
    }

    let token_hex = state
        .dashboard_invitation_service
        .invite(&body.email, &body.role, claims.sub)
        .await?;

    let inviter =
        employee_repository::find_by_id(&state.dashboard_invitation_service.pool, claims.sub)
            .await
            .map_err(|e| {
                tracing::error!("failed to find inviter: {e}");
                AppError::InternalServerError
            })?
            .ok_or(AppError::Unauthorized)?;

    // Send invitation email asynchronously
    let app_name = state.dashboard_invitation_service.app_name.clone();
    let base_url = state.dashboard_invitation_service.base_url.clone();
    let email_sender = state.dashboard_invitation_service.email_sender.clone();
    let email = body.email.clone();
    let role = body.role.clone();

    tokio::spawn(async move {
        let (subject, text_body, html_body) = crate::email::build_invitation_email(
            &app_name,
            &base_url,
            &inviter.email,
            &role,
            &token_hex,
        );

        if let Err(e) = email_sender
            .send(&email, &subject, &text_body, &html_body)
            .await
        {
            tracing::error!("failed to send invitation email to {email}: {e}");
        }
    });

    Ok(Json(InviteResponse {
        message: format!("Invitation sent to {}", body.email),
        email: body.email,
        role: body.role,
    }))
}
