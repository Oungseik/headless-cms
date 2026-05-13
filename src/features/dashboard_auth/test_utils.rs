use super::service_impl::{DashboardAuthServiceImpl, TokenConfig};

pub async fn setup_service() -> DashboardAuthServiceImpl {
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
    }
}
