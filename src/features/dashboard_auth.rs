//! Dashboard authentication endpoints (login, register, logout, token refresh).

pub mod login;
pub mod logout;
pub mod me;
pub mod refresh;
pub mod register;
pub mod resend_verification_email;
pub mod service;
pub mod service_impl;
#[cfg(test)]
pub mod test_utils;
pub mod test_verify_all;
pub mod verify_email;

use std::sync::Arc;

use tower_governor::governor::GovernorConfigBuilder;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{app::AppState, config::AppEnv};

/// Returns the dashboard auth router.
pub fn router(
    app_env: &AppEnv,
    rate_limit_enabled: bool,
    login_rate_limit_per_second: u64,
    login_rate_limit_burst: u32,
) -> OpenApiRouter<Arc<AppState>> {
    let login_governor_conf = if rate_limit_enabled {
        GovernorConfigBuilder::default()
            .per_second(login_rate_limit_per_second)
            .burst_size(login_rate_limit_burst)
            .use_headers()
            .finish()
    } else {
        None
    };

    let login_route = OpenApiRouter::new().routes(routes!(login::handler));

    let login_route = if let Some(governor_conf) = login_governor_conf {
        login_route.layer(tower_governor::GovernorLayer::new(Arc::new(governor_conf)))
    } else {
        login_route
    };

    let router = OpenApiRouter::new()
        .routes(routes!(register::handler))
        .merge(login_route)
        .routes(routes!(logout::handler))
        .routes(routes!(refresh::handler))
        .routes(routes!(verify_email::handler))
        .routes(routes!(resend_verification_email::handler))
        .routes(routes!(me::handler));

    if *app_env == AppEnv::Testing {
        router.routes(routes!(test_verify_all::handler))
    } else {
        router
    }
}
