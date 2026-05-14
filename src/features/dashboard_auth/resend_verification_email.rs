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

#[derive(Debug, Deserialize, ToSchema)]
pub struct ResendVerificationEmailRequest {
    pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct ResendVerificationEmailResponse {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/email/verification/resend",
    operation_id = "dashboard_auth_resend_verification_email",
    request_body = ResendVerificationEmailRequest,
    responses(
        (status = 200, description = "verification email sent", body = ResendVerificationEmailResponse),
        (status = 404, description = "account not found", body = ErrorResponse),
        (status = 409, description = "email already verified", body = ErrorResponse),
    ),
    tag = "Dashboard Auth",
)]
#[tracing::instrument(skip(state))]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ResendVerificationEmailRequest>,
) -> AppResult<(StatusCode, Json<ResendVerificationEmailResponse>)> {
    let token_hex = state
        .dashboard_auth_service
        .resend_verification_email(&body.email)
        .await?;

    let app_name = state.dashboard_auth_service.app_name.clone();
    let base_url = state.dashboard_auth_service.base_url.clone();
    let email_sender = state.dashboard_auth_service.email_sender.clone();
    let email = body.email;

    tokio::spawn(async move {
        let (subject, text_body, html_body) =
            crate::email::build_verification_email(&app_name, &base_url, &token_hex);

        if let Err(e) = email_sender
            .send(&email, &subject, &text_body, &html_body)
            .await
        {
            tracing::error!("failed to send verification email to {email}: {e}");
        }
    });

    Ok((
        StatusCode::OK,
        Json(ResendVerificationEmailResponse {
            message: "Verification email sent".to_string(),
        }),
    ))
}
