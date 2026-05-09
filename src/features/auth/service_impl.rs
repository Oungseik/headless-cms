use std::sync::Arc;

use async_trait::async_trait;
use bcrypt::{DEFAULT_COST, hash, verify};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use sha2::{Digest, Sha256};

use super::email_service::EmailService;
use super::service::{
    AuthResponse, AuthService, AuthServiceError, RefreshResponse, RegisterResponse,
    ResendVerificationResponse, UserResponse, VerifyEmailResponse,
};
use crate::auth::jwt;
use crate::config::Config;

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(Clone, Debug)]
pub struct AuthServiceImpl {
    pub db: DatabaseConnection,
    pub email_service: Arc<dyn EmailService>,
    pub config: Arc<Config>,
}

#[async_trait]
impl AuthService for AuthServiceImpl {
    async fn register(
        &self,
        email: &str,
        password: &str,
        role: &str,
    ) -> Result<RegisterResponse, AuthServiceError> {
        let txn = self.db.begin().await.map_err(AuthServiceError::Database)?;

        let existing = entity::user::Entity::find()
            .filter(entity::user::Column::Email.eq(email))
            .one(&txn)
            .await
            .map_err(AuthServiceError::Database)?;

        if existing.is_some() {
            return Err(AuthServiceError::Conflict(
                "Email already registered".to_string(),
            ));
        }

        let password_hash = hash(password, DEFAULT_COST).map_err(|_| {
            AuthServiceError::Database(sea_orm::DbErr::Custom("password hashing failed".into()))
        })?;

        let now = chrono::Utc::now().naive_utc();

        let user_model = entity::user::ActiveModel {
            email: Set(email.to_string()),
            password_hash: Set(password_hash),
            role: Set(role.to_string()),
            is_active: Set(true),
            email_verified_at: Set(None),
            updated_at: Set(now),
            created_at: Set(now),
            ..Default::default()
        };

        let user = user_model
            .insert(&txn)
            .await
            .map_err(AuthServiceError::Database)?;

        let raw_token = uuid::Uuid::new_v4().to_string();
        let token_hash = hash_token(&raw_token);
        let now_with_tz = chrono::Utc::now().fixed_offset();
        let expires_at = now_with_tz
            + chrono::Duration::seconds(self.config.email_verification_token_ttl as i64);

        let verification_model = entity::email_verification_token::ActiveModel {
            user_id: Set(user.id),
            token_hash: Set(token_hash),
            expires_at: Set(expires_at),
            created_at: Set(now_with_tz),
            ..Default::default()
        };
        verification_model
            .insert(&txn)
            .await
            .map_err(AuthServiceError::Database)?;

        txn.commit().await.map_err(AuthServiceError::Database)?;

        let verification_link = format!(
            "{}/api/v1/auth/verify-email?token={}",
            self.config.base_url, raw_token
        );

        if let Err(e) = self
            .email_service
            .send_verification_email(email, &verification_link)
            .await
        {
            tracing::warn!(error = %e, "failed to send verification email");
        }

        Ok(RegisterResponse {
            message: "Please check your email to verify your account.".to_string(),
        })
    }

    async fn login(&self, email: &str, password: &str) -> Result<AuthResponse, AuthServiceError> {
        let user = entity::user::Entity::find()
            .filter(entity::user::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or_else(|| AuthServiceError::NotFound("Invalid email or password".to_string()))?;

        let valid = verify(password, &user.password_hash)
            .map_err(|_| AuthServiceError::Unauthorized("Invalid email or password".to_string()))?;
        if !valid {
            return Err(AuthServiceError::Unauthorized(
                "Invalid email or password".to_string(),
            ));
        }

        if user.email_verified_at.is_none() {
            return Err(AuthServiceError::NotVerified(
                "Email not verified. Please check your email.".to_string(),
            ));
        }

        if !user.is_active {
            return Err(AuthServiceError::Unauthorized(
                "Account is deactivated".to_string(),
            ));
        }

        let access_token = jwt::generate_access_token(user.id, &user.role).map_err(|_| {
            AuthServiceError::Database(sea_orm::DbErr::Custom("token generation failed".into()))
        })?;
        let refresh_token = jwt::generate_refresh_token(user.id).map_err(|_| {
            AuthServiceError::Database(sea_orm::DbErr::Custom("token generation failed".into()))
        })?;

        let token_hash = hash_token(&refresh_token);
        let now = chrono::Utc::now().naive_utc();
        let expires_at = now + chrono::Duration::seconds(self.config.refresh_token_ttl as i64);

        let rt_model = entity::refresh_token::ActiveModel {
            user_id: Set(user.id),
            token_hash: Set(token_hash),
            expires_at: Set(expires_at),
            revoked_at: Set(None),
            created_at: Set(now),
            ..Default::default()
        };
        rt_model
            .insert(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        Ok(AuthResponse {
            user: UserResponse::from(user),
            access_token,
            refresh_token,
        })
    }

    async fn verify_email(&self, raw_token: &str) -> Result<VerifyEmailResponse, AuthServiceError> {
        let txn = self.db.begin().await.map_err(AuthServiceError::Database)?;

        let token_hash = hash_token(raw_token);

        let stored = entity::email_verification_token::Entity::find()
            .filter(entity::email_verification_token::Column::TokenHash.eq(&token_hash))
            .one(&txn)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or_else(|| AuthServiceError::NotFound("Invalid verification token".to_string()))?;

        let now = chrono::Utc::now();
        if stored.expires_at < now {
            let active: entity::email_verification_token::ActiveModel = stored.into();
            active
                .delete(&txn)
                .await
                .map_err(AuthServiceError::Database)?;
            return Err(AuthServiceError::NotFound(
                "Verification token expired. Please request a new one.".to_string(),
            ));
        }

        let user = entity::user::Entity::find_by_id(stored.user_id)
            .one(&txn)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or_else(|| AuthServiceError::NotFound("User not found".to_string()))?;

        let mut user_active: entity::user::ActiveModel = user.into();
        user_active.email_verified_at = Set(Some(now.into()));
        user_active
            .update(&txn)
            .await
            .map_err(AuthServiceError::Database)?;

        let token_active: entity::email_verification_token::ActiveModel = stored.into();
        token_active
            .delete(&txn)
            .await
            .map_err(AuthServiceError::Database)?;

        txn.commit().await.map_err(AuthServiceError::Database)?;

        Ok(VerifyEmailResponse {
            message: "Email verified successfully. You can now log in.".to_string(),
        })
    }

    async fn resend_verification(
        &self,
        email: &str,
    ) -> Result<ResendVerificationResponse, AuthServiceError> {
        let generic_message = ResendVerificationResponse {
            message: "If an account with that email exists and is not yet verified, we've sent a verification email.".to_string(),
        };

        let user = entity::user::Entity::find()
            .filter(entity::user::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        let user = match user {
            Some(u) => u,
            None => return Ok(generic_message),
        };

        if user.email_verified_at.is_some() {
            return Ok(generic_message);
        }

        // Delete existing verification tokens
        entity::email_verification_token::Entity::delete_many()
            .filter(entity::email_verification_token::Column::UserId.eq(user.id))
            .exec(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        // Generate new verification token
        let raw_token = uuid::Uuid::new_v4().to_string();
        let token_hash = hash_token(&raw_token);
        let now_with_tz = chrono::Utc::now().fixed_offset();
        let expires_at = now_with_tz
            + chrono::Duration::seconds(self.config.email_verification_token_ttl as i64);

        let verification_model = entity::email_verification_token::ActiveModel {
            user_id: Set(user.id),
            token_hash: Set(token_hash),
            expires_at: Set(expires_at),
            created_at: Set(now_with_tz),
            ..Default::default()
        };
        verification_model
            .insert(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        let verification_link = format!(
            "{}/api/v1/auth/verify-email?token={}",
            self.config.base_url, raw_token
        );

        if let Err(e) = self
            .email_service
            .send_verification_email(email, &verification_link)
            .await
        {
            tracing::warn!(error = %e, "failed to send verification email");
        }

        Ok(generic_message)
    }

    async fn refresh(&self, token: &str) -> Result<RefreshResponse, AuthServiceError> {
        let claims = jwt::validate_refresh_token(token)
            .map_err(|_| AuthServiceError::Unauthorized("Invalid refresh token".to_string()))?;

        let token_hash = hash_token(token);
        let now = chrono::Utc::now().naive_utc();

        let stored = entity::refresh_token::Entity::find()
            .filter(entity::refresh_token::Column::TokenHash.eq(&token_hash))
            .one(&self.db)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or_else(|| AuthServiceError::Unauthorized("Invalid refresh token".to_string()))?;

        if stored.revoked_at.is_some() {
            return Err(AuthServiceError::Unauthorized(
                "Refresh token revoked".to_string(),
            ));
        }

        if stored.expires_at < now {
            return Err(AuthServiceError::Unauthorized(
                "Refresh token expired".to_string(),
            ));
        }

        let mut active: entity::refresh_token::ActiveModel = stored.into();
        active.revoked_at = Set(Some(now));
        active
            .update(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        let user_id: i32 = claims
            .sub
            .parse()
            .map_err(|_| AuthServiceError::Unauthorized("Invalid token subject".to_string()))?;
        let user = entity::user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or_else(|| AuthServiceError::NotFound("User not found".to_string()))?;

        if !user.is_active {
            return Err(AuthServiceError::Unauthorized(
                "Account is deactivated".to_string(),
            ));
        }

        let access_token = jwt::generate_access_token(user.id, &user.role).map_err(|_| {
            AuthServiceError::Database(sea_orm::DbErr::Custom("token generation failed".into()))
        })?;
        let new_refresh_token = jwt::generate_refresh_token(user.id).map_err(|_| {
            AuthServiceError::Database(sea_orm::DbErr::Custom("token generation failed".into()))
        })?;

        let new_token_hash = hash_token(&new_refresh_token);
        let expires_at = now + chrono::Duration::seconds(self.config.refresh_token_ttl as i64);

        let rt_model = entity::refresh_token::ActiveModel {
            user_id: Set(user.id),
            token_hash: Set(new_token_hash),
            expires_at: Set(expires_at),
            revoked_at: Set(None),
            created_at: Set(now),
            ..Default::default()
        };
        rt_model
            .insert(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        Ok(RefreshResponse {
            access_token,
            refresh_token: new_refresh_token,
        })
    }

    async fn logout(&self, token: &str) -> Result<(), AuthServiceError> {
        let _claims = jwt::validate_refresh_token(token)
            .map_err(|_| AuthServiceError::Unauthorized("Invalid refresh token".to_string()))?;

        let token_hash = hash_token(token);
        let now = chrono::Utc::now().naive_utc();

        let stored = entity::refresh_token::Entity::find()
            .filter(entity::refresh_token::Column::TokenHash.eq(&token_hash))
            .one(&self.db)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or_else(|| AuthServiceError::Unauthorized("Invalid refresh token".to_string()))?;

        if stored.revoked_at.is_some() {
            return Ok(());
        }

        let mut active: entity::refresh_token::ActiveModel = stored.into();
        active.revoked_at = Set(Some(now));
        active
            .update(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        Ok(())
    }
}
