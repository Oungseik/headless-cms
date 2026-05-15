use std::sync::Arc;

use super::service_impl::{DashboardAuthServiceImpl, TokenConfig};
use crate::email::NoopEmailSender;

pub async fn setup_service() -> DashboardAuthServiceImpl<NoopEmailSender> {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory sqlite");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");
    DashboardAuthServiceImpl {
        pool,
        bcrypt_cost: 4,
        email_verification_token_ttl: 86400,
        token_config: TokenConfig {
            jwt_secret: "test-secret-key".to_string(),
            access_token_ttl: 900,
            refresh_token_ttl: 604_800,
        },
        email_sender: Arc::new(NoopEmailSender),
        app_name: "Test App".to_string(),
        base_url: "http://localhost:3000".to_string(),
    }
}
