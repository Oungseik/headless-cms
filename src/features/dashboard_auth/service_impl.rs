use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::service::{DashboardAuthService, DashboardAuthServiceError};
use crate::auth;
use crate::repositories::{employee_email_verification_token_repository, employee_repository};

#[derive(Clone)]
pub struct DashboardAuthServiceImpl {
    pub pool: SqlitePool,
    pub bcrypt_cost: u32,
    pub email_verification_token_ttl: u64,
}

#[async_trait]
impl DashboardAuthService for DashboardAuthServiceImpl {
    async fn register<'a>(
        &'a self,
        email: &'a str,
        password: &'a str,
    ) -> Result<(), DashboardAuthServiceError> {
        if password.len() < 8 {
            return Err(DashboardAuthServiceError::WeakPassword);
        }

        let mut txn = self.pool.begin().await?;

        let count = employee_repository::count_all(&mut *txn).await?;

        if count > 0 {
            txn.rollback().await?;
            return Err(DashboardAuthServiceError::OwnerAlreadyExists);
        }

        let password_hash = auth::password::hash(password, self.bcrypt_cost)
            .map_err(DashboardAuthServiceError::PasswordHashing)?;

        let employee_id = Uuid::now_v7();
        let now = Utc::now();

        employee_repository::insert(
            &mut *txn,
            employee_id,
            email,
            &password_hash,
            "owner",
            true,
            None,
            now,
            now,
        )
        .await?;

        let token_id = Uuid::now_v7();
        let (_, token_hash) = auth::token::generate();
        let ttl = i64::try_from(self.email_verification_token_ttl).unwrap_or(i64::MAX);
        let expires_at = now + chrono::Duration::seconds(ttl);

        employee_email_verification_token_repository::insert(
            &mut *txn,
            token_id,
            employee_id,
            &token_hash,
            expires_at,
            now,
        )
        .await?;

        txn.commit().await?;

        Ok(())
    }
}
