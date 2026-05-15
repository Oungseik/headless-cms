use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    app::{
        AppState,
        error::{AppResult, ErrorResponse},
    },
    email::EmailSender,
    features::dashboard_auth::service::DashboardAuthService,
};

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

impl std::fmt::Debug for RegisterRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisterRequest")
            .field("email", &self.email)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

#[derive(Serialize, ToSchema)]
pub struct RegisterResponse {
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/register",
    operation_id = "dashboard_auth_register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "owner registered", body = RegisterResponse),
        (status = 400, description = "weak password", body = ErrorResponse),
        (status = 409, description = "owner already exists", body = ErrorResponse),
    ),
    tag = "Dashboard Auth",
)]
#[tracing::instrument(skip(state))]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<RegisterResponse>)> {
    let token_hex = state
        .dashboard_auth_service
        .register(&body.email, &body.password)
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
        StatusCode::CREATED,
        Json(RegisterResponse {
            message: "Please check your email to verify your account.".to_string(),
        }),
    ))
}
