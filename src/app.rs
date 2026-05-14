// allow: false positive from utoipa derive macro expansion
#![allow(clippy::needless_for_each)]

pub mod error;

use std::sync::Arc;

use axum::{
    Router,
    http::{HeaderValue, Method},
};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use sqlx::SqlitePool;
use tower_governor::governor::GovernorConfigBuilder;
use tower_http::cors::CorsLayer;
use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    app::error::{AppError, AppResult},
    config::get_config,
    email, features,
    features::dashboard_auth::service_impl::{DashboardAuthServiceImpl, TokenConfig},
};

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
#[derive(Debug)]
pub struct AppState {
    pub dashboard_auth_service: DashboardAuthServiceImpl,
}

/// Builds the complete Axum [`Router`] with all routes, middleware, and CORS.
///
/// # Errors
///
/// Returns [`AppError`] if CORS origin parsing fails or database connection fails.
pub async fn create_app() -> AppResult<Router> {
    let config = get_config();

    let pool = SqlitePool::connect(&config.database_url)
        .await
        .map_err(|e| {
            tracing::error!("failed to connect to database: {e}");
            AppError::InternalServerError
        })?;

    sqlx::migrate!().run(&pool).await.map_err(|e| {
        tracing::error!("failed to run migrations: {e}");
        AppError::InternalServerError
    })?;

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

    let governor_conf = if config.rate_limit_enabled {
        GovernorConfigBuilder::default()
            .per_second(config.rate_limit_per_second)
            .burst_size(config.rate_limit_burst)
            .use_headers()
            .finish()
    } else {
        None
    };

    let email_sender = email::build_email_sender(config);

    let dashboard_auth_service = DashboardAuthServiceImpl {
        pool,
        bcrypt_cost: config.bcrypt_cost,
        email_verification_token_ttl: config.email_verification_token_ttl,
        token_config: TokenConfig {
            jwt_secret: config.jwt_secret.clone(),
            access_token_ttl: config.access_token_ttl,
            refresh_token_ttl: config.refresh_token_ttl,
        },
        email_sender,
        app_name: config.app_name.clone(),
        base_url: config.base_url.clone(),
    };

    let state = Arc::new(AppState {
        dashboard_auth_service,
    });

    let health_route = features::health::router();
    let dashboard_auth_route = features::dashboard_auth::router(&config.app_env);

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/health", health_route)
        .nest("/api/v1/dashboard/auth", dashboard_auth_route)
        .with_state(state)
        .split_for_parts();

    let swagger = SwaggerUi::new("/api-docs/swagger-ui").url("/api-docs/openapi.json", api);
    let router = router.merge(swagger).layer(cors);

    let router = if let Some(governor_conf) = governor_conf {
        router.layer(tower_governor::GovernorLayer::new(Arc::new(governor_conf)))
    } else {
        router
    };

    let router = router
        .layer(OtelInResponseLayer)
        .layer(OtelAxumLayer::default());

    Ok(router)
}
