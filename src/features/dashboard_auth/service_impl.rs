use async_trait::async_trait;
use chrono::Utc;
use entity::employee;
use entity::employee_email_verification_token;
use rand::Rng;
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, Set, TransactionTrait};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::service::{DashboardAuthService, DashboardAuthServiceError};

#[derive(Clone)]
pub struct DashboardAuthServiceImpl {
    pub db: DatabaseConnection,
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

        let txn = self.db.begin().await?;

        let count = employee::Entity::find().count(&txn).await?;

        if count > 0 {
            txn.rollback().await?;
            return Err(DashboardAuthServiceError::OwnerAlreadyExists);
        }

        let password_hash = bcrypt::hash(password, self.bcrypt_cost)
            .map_err(DashboardAuthServiceError::PasswordHashing)?;

        let employee_id = Uuid::now_v7();
        let now = Utc::now();

        let employee = employee::ActiveModel {
            id: Set(employee_id),
            email: Set(email.to_owned()),
            password_hash: Set(password_hash),
            role: Set("owner".to_string()),
            is_active: Set(true),
            email_verified_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        employee::Entity::insert(employee).exec(&txn).await?;

        let token_bytes: [u8; 32] = rand::thread_rng().r#gen();
        let token_hash = hex::encode(Sha256::digest(token_bytes));
        let token_id = Uuid::now_v7();
        let ttl = i64::try_from(self.email_verification_token_ttl).unwrap_or(i64::MAX);
        let expires_at = now + chrono::Duration::seconds(ttl);

        let token = employee_email_verification_token::ActiveModel {
            id: Set(token_id),
            employee_id: Set(employee_id),
            token_hash: Set(token_hash),
            expires_at: Set(expires_at),
            created_at: Set(now),
        };

        employee_email_verification_token::Entity::insert(token)
            .exec(&txn)
            .await?;

        txn.commit().await?;

        Ok(())
    }
}
