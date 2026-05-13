use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::service::{DashboardAuthService, DashboardAuthServiceError, LoginResult};
use crate::{
    auth,
    repositories::{
        employee_email_verification_token_repository, employee_refresh_token_repository,
        employee_repository,
    },
};

/// Configuration for JWT and refresh token generation.
#[derive(Clone, Debug)]
pub struct TokenConfig {
    /// Secret key used to sign JWTs.
    pub jwt_secret: String,
    /// Access token time-to-live in seconds.
    pub access_token_ttl: u64,
    /// Refresh token time-to-live in seconds.
    pub refresh_token_ttl: u64,
}

/// Implementation of [`DashboardAuthService`] backed by ``SQLite``.
#[derive(Clone, Debug)]
pub struct DashboardAuthServiceImpl {
    /// Database connection pool.
    pub pool: SqlitePool,
    /// bcrypt cost factor for password hashing.
    pub bcrypt_cost: u32,
    /// Email verification token TTL in seconds.
    pub email_verification_token_ttl: u64,
    /// Token generation configuration.
    pub token_config: TokenConfig,
}

impl DashboardAuthServiceImpl {
    /// Generates an access token (JWT) and a refresh token for the given
    /// employee, persisting the refresh token hash within the provided transaction.
    async fn generate_tokens(
        &self,
        txn: &mut sqlx::SqliteConnection,
        employee_id: Uuid,
        role: &str,
    ) -> Result<(String, Vec<u8>), DashboardAuthServiceError> {
        let access_token = auth::jwt::encode(
            employee_id,
            role,
            self.token_config.jwt_secret.as_bytes(),
            self.token_config.access_token_ttl,
        )?;

        let (raw_refresh_token, refresh_token_hash) = auth::token::generate();
        let now = Utc::now();
        let ttl = i64::try_from(self.token_config.refresh_token_ttl).unwrap_or(i64::MAX);
        let refresh_expires_at = now + chrono::Duration::seconds(ttl);
        let refresh_token_id = Uuid::now_v7();

        employee_refresh_token_repository::insert(
            txn,
            refresh_token_id,
            employee_id,
            &refresh_token_hash,
            refresh_expires_at,
            now,
        )
        .await?;

        Ok((access_token, raw_refresh_token.to_vec()))
    }
}

impl DashboardAuthService for DashboardAuthServiceImpl {
    async fn register(&self, email: &str, password: &str) -> Result<(), DashboardAuthServiceError> {
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
            employee_repository::CreateEmployee {
                id: employee_id,
                email,
                password_hash: &password_hash,
                role: "owner",
                is_active: true,
                email_verified_at: None,
                created_at: now,
                updated_at: now,
            },
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

    async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<LoginResult, DashboardAuthServiceError> {
        let mut txn = self.pool.begin().await?;

        let employee = employee_repository::find_by_email(&mut *txn, email)
            .await?
            .ok_or(DashboardAuthServiceError::InvalidCredentials)?;

        if !employee.is_active {
            return Err(DashboardAuthServiceError::AccountInactive);
        }

        if employee.email_verified_at.is_none() {
            return Err(DashboardAuthServiceError::EmailNotVerified);
        }

        let password_valid = auth::password::verify(password, &employee.password_hash)
            .map_err(DashboardAuthServiceError::PasswordHashing)?;

        if !password_valid {
            return Err(DashboardAuthServiceError::InvalidCredentials);
        }

        let (access_token, raw_refresh_token) = self
            .generate_tokens(&mut txn, employee.id, &employee.role)
            .await?;

        txn.commit().await?;

        Ok(LoginResult {
            access_token,
            refresh_token: raw_refresh_token,
            expires_in: self.token_config.access_token_ttl,
        })
    }

    async fn logout(&self, token: &str) -> Result<(), DashboardAuthServiceError> {
        let raw = hex::decode(token).map_err(|_| DashboardAuthServiceError::InvalidCredentials)?;
        let token_hash = auth::token::hash(&raw);
        let now = Utc::now();
        employee_refresh_token_repository::revoke(&self.pool, &token_hash, now).await?;
        Ok(())
    }

    async fn refresh(&self, token: &str) -> Result<LoginResult, DashboardAuthServiceError> {
        let raw = hex::decode(token).map_err(|_| DashboardAuthServiceError::InvalidCredentials)?;
        let token_hash = auth::token::hash(&raw);
        let now = Utc::now();

        let mut txn = self.pool.begin().await?;

        let refresh_token = employee_refresh_token_repository::find_by_hash(&mut *txn, &token_hash)
            .await?
            .ok_or(DashboardAuthServiceError::InvalidCredentials)?;

        if refresh_token.revoked_at.is_some() {
            return Err(DashboardAuthServiceError::InvalidCredentials);
        }

        if refresh_token.expires_at < now {
            return Err(DashboardAuthServiceError::InvalidCredentials);
        }

        let employee = employee_repository::find_by_id(&mut *txn, refresh_token.employee_id)
            .await?
            .ok_or(DashboardAuthServiceError::InvalidCredentials)?;

        if !employee.is_active {
            return Err(DashboardAuthServiceError::AccountInactive);
        }

        employee_refresh_token_repository::revoke(&mut *txn, &token_hash, now).await?;

        let (access_token, raw_refresh_token) = self
            .generate_tokens(&mut txn, employee.id, &employee.role)
            .await?;

        txn.commit().await?;

        Ok(LoginResult {
            access_token,
            refresh_token: raw_refresh_token,
            expires_in: self.token_config.access_token_ttl,
        })
    }

    async fn verify_all(&self) -> Result<(), DashboardAuthServiceError> {
        let now = Utc::now();
        let mut txn = self.pool.begin().await?;

        employee_repository::update_all_email_verified_at(&mut *txn, now).await?;
        employee_email_verification_token_repository::delete_all(&mut *txn).await?;

        txn.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::dashboard_auth::{
        service::DashboardAuthServiceError, test_utils::setup_service,
    };

    #[tokio::test]
    async fn register_should_fail_when_password_too_short() {
        let service = setup_service().await;
        let result = service.register("owner@example.com", "short").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DashboardAuthServiceError::WeakPassword
        ));
    }

    #[tokio::test]
    async fn register_should_succeed_with_valid_input() {
        let service = setup_service().await;
        let result = service.register("owner@example.com", "password1234").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn register_should_fail_when_owner_already_exists() {
        let service = setup_service().await;
        service
            .register("owner@example.com", "password1234")
            .await
            .expect("first registration should succeed");

        let result = service.register("other@example.com", "password1234").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DashboardAuthServiceError::OwnerAlreadyExists
        ));
    }

    #[tokio::test]
    async fn login_should_fail_with_invalid_email() {
        let service = setup_service().await;
        let result = service
            .login("nonexistent@example.com", "password1234")
            .await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DashboardAuthServiceError::InvalidCredentials
        ));
    }

    #[tokio::test]
    async fn login_should_fail_with_wrong_password() {
        let service = setup_service().await;
        service
            .register("owner@example.com", "password1234")
            .await
            .expect("registration should succeed");

        service
            .verify_all()
            .await
            .expect("verify_all should succeed");

        let result = service.login("owner@example.com", "wrongpassword").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DashboardAuthServiceError::InvalidCredentials
        ));
    }

    #[tokio::test]
    async fn login_should_fail_when_email_not_verified() {
        let service = setup_service().await;
        service
            .register("owner@example.com", "password1234")
            .await
            .expect("registration should succeed");

        let result = service.login("owner@example.com", "password1234").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DashboardAuthServiceError::EmailNotVerified
        ));
    }

    #[tokio::test]
    async fn login_should_fail_when_account_inactive() {
        let service = setup_service().await;
        service
            .register("owner@example.com", "password1234")
            .await
            .expect("registration should succeed");

        service
            .verify_all()
            .await
            .expect("verify_all should succeed");

        let mut txn = service.pool.begin().await.unwrap();
        sqlx::query("UPDATE employee SET is_active = FALSE WHERE email = ?")
            .bind("owner@example.com")
            .execute(&mut *txn)
            .await
            .unwrap();
        txn.commit().await.unwrap();

        let result = service.login("owner@example.com", "password1234").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DashboardAuthServiceError::AccountInactive
        ));
    }

    #[tokio::test]
    async fn login_should_succeed_and_return_tokens() {
        let service = setup_service().await;
        service
            .register("owner@example.com", "password1234")
            .await
            .expect("registration should succeed");

        service
            .verify_all()
            .await
            .expect("verify_all should succeed");

        let result = service.login("owner@example.com", "password1234").await;
        assert!(result.is_ok());

        let login_result = result.unwrap();
        assert!(!login_result.access_token.is_empty());
        assert!(!login_result.refresh_token.is_empty());
        assert_eq!(login_result.expires_in, 900);
    }

    #[tokio::test]
    async fn logout_should_succeed_with_valid_token() {
        let service = setup_service().await;
        service
            .register("owner@example.com", "password1234")
            .await
            .expect("registration should succeed");

        service
            .verify_all()
            .await
            .expect("verify_all should succeed");

        let login_result = service
            .login("owner@example.com", "password1234")
            .await
            .expect("login should succeed");

        let hex_token = hex::encode(&login_result.refresh_token);
        let result = service.logout(&hex_token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn logout_should_succeed_with_invalid_token() {
        let service = setup_service().await;
        let result = service.logout("not-a-valid-hex").await;
        assert!(matches!(
            result.unwrap_err(),
            DashboardAuthServiceError::InvalidCredentials
        ));
    }

    #[tokio::test]
    async fn logout_should_succeed_with_already_revoked_token() {
        let service = setup_service().await;
        service
            .register("owner@example.com", "password1234")
            .await
            .expect("registration should succeed");

        service
            .verify_all()
            .await
            .expect("verify_all should succeed");

        let login_result = service
            .login("owner@example.com", "password1234")
            .await
            .expect("login should succeed");

        let hex_token = hex::encode(&login_result.refresh_token);

        service
            .logout(&hex_token)
            .await
            .expect("first logout should succeed");

        let result = service.logout(&hex_token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn refresh_should_succeed_with_valid_token() {
        let service = setup_service().await;
        service
            .register("owner@example.com", "password1234")
            .await
            .expect("registration should succeed");

        service
            .verify_all()
            .await
            .expect("verify_all should succeed");

        let login_result = service
            .login("owner@example.com", "password1234")
            .await
            .expect("login should succeed");

        let hex_token = hex::encode(&login_result.refresh_token);
        let result = service.refresh(&hex_token).await;
        assert!(result.is_ok());

        let refresh_result = result.unwrap();
        assert!(!refresh_result.access_token.is_empty());
        assert!(!refresh_result.refresh_token.is_empty());
        assert_eq!(refresh_result.expires_in, 900);
    }

    #[tokio::test]
    async fn refresh_should_fail_with_invalid_token() {
        let service = setup_service().await;
        let result = service.refresh("deadbeef").await;
        assert!(matches!(
            result.unwrap_err(),
            DashboardAuthServiceError::InvalidCredentials
        ));
    }

    #[tokio::test]
    async fn refresh_should_fail_with_revoked_token() {
        let service = setup_service().await;
        service
            .register("owner@example.com", "password1234")
            .await
            .expect("registration should succeed");

        service
            .verify_all()
            .await
            .expect("verify_all should succeed");

        let login_result = service
            .login("owner@example.com", "password1234")
            .await
            .expect("login should succeed");

        let hex_token = hex::encode(&login_result.refresh_token);

        service
            .logout(&hex_token)
            .await
            .expect("logout should succeed");

        let result = service.refresh(&hex_token).await;
        assert!(matches!(
            result.unwrap_err(),
            DashboardAuthServiceError::InvalidCredentials
        ));
    }

    #[tokio::test]
    async fn refresh_should_fail_with_expired_token() {
        let service = setup_service().await;
        service
            .register("owner@example.com", "password1234")
            .await
            .expect("registration should succeed");

        service
            .verify_all()
            .await
            .expect("verify_all should succeed");

        let login_result = service
            .login("owner@example.com", "password1234")
            .await
            .expect("login should succeed");

        let raw = &login_result.refresh_token;
        let token_hash = crate::auth::token::hash(raw);

        let past = Utc::now() - chrono::Duration::hours(1);
        sqlx::query("UPDATE employee_refresh_token SET expires_at = ? WHERE token_hash = ?")
            .bind(past)
            .bind(&token_hash)
            .execute(&service.pool)
            .await
            .unwrap();

        let hex_token = hex::encode(raw);
        let result = service.refresh(&hex_token).await;
        assert!(matches!(
            result.unwrap_err(),
            DashboardAuthServiceError::InvalidCredentials
        ));
    }

    #[tokio::test]
    async fn refresh_should_fail_when_account_inactive() {
        let service = setup_service().await;
        service
            .register("owner@example.com", "password1234")
            .await
            .expect("registration should succeed");

        service
            .verify_all()
            .await
            .expect("verify_all should succeed");

        let login_result = service
            .login("owner@example.com", "password1234")
            .await
            .expect("login should succeed");

        let mut txn = service.pool.begin().await.unwrap();
        sqlx::query("UPDATE employee SET is_active = FALSE WHERE email = ?")
            .bind("owner@example.com")
            .execute(&mut *txn)
            .await
            .unwrap();
        txn.commit().await.unwrap();

        let hex_token = hex::encode(&login_result.refresh_token);
        let result = service.refresh(&hex_token).await;
        assert!(matches!(
            result.unwrap_err(),
            DashboardAuthServiceError::AccountInactive
        ));
    }

    #[tokio::test]
    async fn refresh_should_revoke_old_token() {
        let service = setup_service().await;
        service
            .register("owner@example.com", "password1234")
            .await
            .expect("registration should succeed");

        service
            .verify_all()
            .await
            .expect("verify_all should succeed");

        let login_result = service
            .login("owner@example.com", "password1234")
            .await
            .expect("login should succeed");

        let hex_token = hex::encode(&login_result.refresh_token);

        service
            .refresh(&hex_token)
            .await
            .expect("first refresh should succeed");

        let result = service.refresh(&hex_token).await;
        assert!(matches!(
            result.unwrap_err(),
            DashboardAuthServiceError::InvalidCredentials
        ));
    }
}
