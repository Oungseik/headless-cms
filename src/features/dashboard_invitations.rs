//! Dashboard invitation endpoints.

pub mod invite;
pub mod service;
pub mod service_impl;

use std::sync::Arc;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::app::AppState;

/// Returns the dashboard invitations router.
pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(invite::handler))
}
