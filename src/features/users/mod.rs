pub mod get_by_id;
pub mod service;
pub mod service_impl;

#[cfg(test)]
pub mod mock_service;

use std::sync::Arc;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::app::AppState;

pub fn router() -> OpenApiRouter<Arc<AppState>> {
    OpenApiRouter::new().routes(routes!(get_by_id::handler))
}
