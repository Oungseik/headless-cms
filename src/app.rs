// allow: false positive from utoipa derive macro expansion
#![allow(clippy::needless_for_each)]

pub mod error;

use std::sync::Arc;

use axum::Router;
use axum::http::{HeaderValue, Method};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use tower_http::cors::CorsLayer;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::app::error::{AppError, AppResult};
use crate::config::get_config;
use crate::features;

/// `OpenAPI` documentation specification.
#[derive(OpenApi)]
#[openapi(modifiers(&SecurityAddon))]
pub struct ApiDoc;

/// Adds JWT bearer security scheme to the `OpenAPI` spec.
pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(component) = openapi.components.as_mut() {
            component.add_security_scheme(
                "Authorization",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            );
        }
    }
}

/// Shared application state passed to all route handlers.
///
/// Holds `Arc<dyn <Domain>Service>` for each domain, enabling test-time
/// substitution via mocks. `DatabaseConnection` is NOT stored here — it lives
/// only inside each `*ServiceImpl`.
#[derive(Clone)]
pub struct AppState {}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("db", &"SqlitePool")
            .field("dashboard_auth_service", &"Arc<dyn DashboardAuthService>")
            .finish()
    }
}

/// Builds the complete Axum [`Router`] with all routes, middleware, and CORS.
///
/// # Errors
///
/// Returns [`AppError`] if CORS origin parsing fails or database connection fails.
pub async fn create_app() -> AppResult<Router> {
    let config = get_config();
    let cors_origins: Vec<_> = config
        .allowed_origins
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            HeaderValue::from_str(s).map_err(|e| {
                AppError::BadRequest(format!("invalid origin in ALLOWED_ORIGINS: {e}"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(cors_origins);

    let state = Arc::new(AppState {});

    let health_route = features::health::router();
    let dashboard_auth_route = features::dashboard_auth::router();

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/health", health_route)
        .nest("/api/v1/dashboard/auth", dashboard_auth_route)
        .with_state(state)
        .split_for_parts();

    let swagger = SwaggerUi::new("/api-docs/swagger-ui").url("/api-docs/openapi.json", api);
    let router = router
        .merge(swagger)
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default())
        .layer(cors);

    Ok(router)
}
