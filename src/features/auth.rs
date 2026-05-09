pub mod email_service;
pub mod login;
pub mod logout;
pub mod refresh;
pub mod register;
pub mod resend_verification;
pub mod service;
pub mod service_impl;
pub mod verify_email;

#[cfg(test)]
pub mod service_mock;

use std::sync::Arc;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::app::AppState;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(login::handler))
        .routes(routes!(register::handler))
        .routes(routes!(verify_email::handler))
        .routes(routes!(resend_verification::handler))
        .routes(routes!(refresh::handler))
        .routes(routes!(logout::handler))
}
