pub mod email_service;
pub mod register;
pub mod resend_verification;
pub mod service;
pub mod service_impl;
#[cfg(test)]
pub mod service_mock;
pub mod verify_email;

use std::sync::Arc;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::app::AppState;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new()
        .routes(routes!(register::handler))
        .routes(routes!(verify_email::handler))
        .routes(routes!(resend_verification::handler))
}
