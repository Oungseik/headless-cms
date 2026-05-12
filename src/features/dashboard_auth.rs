//! Dashboard authentication endpoints (login, register, logout, token refresh).

pub mod register;
pub mod service;
pub mod service_impl;
#[cfg(test)]
pub mod test_utils;
pub mod test_verify_all;

use std::sync::Arc;

use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{app::AppState, config::AppEnv};

/// Returns the dashboard auth router.
pub fn router(app_env: &AppEnv) -> OpenApiRouter<Arc<AppState>> {
    let router = OpenApiRouter::new().routes(routes!(register::handler));

    if *app_env == AppEnv::Testing {
        router.routes(routes!(test_verify_all::handler))
    } else {
        router
    }
}
