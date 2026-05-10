pub mod error;

use std::sync::Arc;

use axum::Router;
use axum::http::Method;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::get_config;
use crate::features;
use crate::features::auth::email_service::ConsoleEmailService;
use crate::features::auth::service::AuthService;
use crate::features::users::service::UserService;

#[derive(OpenApi)]
#[openapi( modifiers(&SecurityAddon))]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(component) = openapi.components.as_mut() {
            component.add_security_scheme(
                "Authorization",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            );

            component.add_security_scheme(
                "auth_token",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("auth_token"))),
            );
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub user_service: Arc<dyn UserService>,
    pub auth_service: Arc<dyn AuthService>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("db", &"SqlitePool")
            .field("user_service", &"Arc<dyn UserService>")
            .field("auth_service", &"Arc<dyn AuthService>")
            .finish()
    }
}

pub async fn create_app() -> Result<Router, sqlx::Error> {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any);

    let db = setup_db().await?;
    let config = get_config();
    let user_service: Arc<dyn UserService> =
        Arc::new(crate::features::users::service_impl::UserServiceImpl { db: db.clone() });
    let email_service = Arc::new(ConsoleEmailService);
    let auth_service: Arc<dyn AuthService> =
        Arc::new(crate::features::auth::service_impl::AuthServiceImpl {
            db: db.clone(),
            email_service,
            config: Arc::new(config.clone()),
        });
    let state = Arc::new(AppState {
        db,
        user_service,
        auth_service,
    });

    let health_route = features::health::router();
    let users_route = features::users::router();
    let auth_route = features::auth::router();

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest("/health", health_route)
        .nest("/api/v1/users", users_route)
        .nest("/api/v1/auth", auth_route)
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

pub async fn setup_db() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect(&get_config().database_url).await?;
    sqlx::query("PRAGMA journal_mode=WAL;")
        .execute(&pool)
        .await?;

    #[cfg(feature = "integration_testing")]
    {
        for m in migration::migrations() {
            for sql in (m.up)() {
                sqlx::raw_sql(&sql).execute(&pool).await?;
            }
        }
    }

    Ok(pool)
}
