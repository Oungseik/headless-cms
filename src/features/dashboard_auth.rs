//! Dashboard authentication endpoints (login, register, logout, token refresh).

pub mod register;
pub mod service;
pub mod service_impl;

use std::sync::Arc;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::app::AppState;

/// Returns the dashboard auth router.
pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(register::handler))
}
