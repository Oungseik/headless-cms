use std::sync::Arc;

use async_trait::async_trait;
use bcrypt::{DEFAULT_COST, hash, verify};
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

use super::email_service::EmailService;
use super::service::{
    AuthResponse, AuthService, AuthServiceError, MeResponse, RefreshResponse, RegisterResponse,
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
    pub db: SqlitePool,
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
        use entity::email_verification_token::EmailVerificationToken;
        use entity::user::User;

        let mut txn = self.db.begin().await.map_err(AuthServiceError::Database)?;

        // Check existing user
        let (sql, values) = Query::select()
            .columns([User::Id])
            .from(User::Table)
            .and_where(Expr::col(User::Email).eq(email))
            .build_sqlx(SqliteQueryBuilder);

        let existing: Option<(i32,)> = sqlx::query_as_with(&sql, values)
            .fetch_optional(&mut *txn)
            .await
            .map_err(AuthServiceError::Database)?;

        if existing.is_some() {
            return Err(AuthServiceError::Conflict(
                "Email already registered".to_string(),
            ));
        }

        let password_hash = hash(password, DEFAULT_COST)
            .map_err(|_| AuthServiceError::Internal("password hashing failed".into()))?;

        let now = chrono::Utc::now().naive_utc();

        // Insert user
        let (sql, values) = Query::insert()
            .into_table(User::Table)
            .columns([
                User::Email,
                User::PasswordHash,
                User::Role,
                User::IsActive,
                User::UpdatedAt,
                User::CreatedAt,
            ])
            .values_panic([
                email.into(),
                password_hash.into(),
                role.into(),
                true.into(),
                now.to_string().into(),
                now.to_string().into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        let res = sqlx::query_with(&sql, values)
            .execute(&mut *txn)
            .await
            .map_err(AuthServiceError::Database)?;

        let user_id: i32 = res.last_insert_rowid() as i32;

        // Insert verification token
        let raw_token = uuid::Uuid::new_v4().to_string();
        let token_hash = hash_token(&raw_token);
        let now_with_tz = chrono::Utc::now().fixed_offset();
        let expires_at = now_with_tz
            + chrono::Duration::seconds(self.config.email_verification_token_ttl.cast_signed());

        let (sql, values) = Query::insert()
            .into_table(EmailVerificationToken::Table)
            .columns([
                EmailVerificationToken::UserId,
                EmailVerificationToken::TokenHash,
                EmailVerificationToken::ExpiresAt,
                EmailVerificationToken::CreatedAt,
            ])
            .values_panic([
                user_id.into(),
                token_hash.into(),
                expires_at.to_rfc3339().into(),
                now_with_tz.to_rfc3339().into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *txn)
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
        use entity::refresh_token::RefreshToken;
        use entity::user::{User, UserRow};

        let (sql, values) = Query::select()
            .columns([
                User::Id,
                User::Email,
                User::PasswordHash,
                User::Role,
                User::IsActive,
                User::EmailVerifiedAt,
                User::CreatedAt,
                User::UpdatedAt,
            ])
            .from(User::Table)
            .and_where(Expr::col(User::Email).eq(email))
            .build_sqlx(SqliteQueryBuilder);

        let user = sqlx::query_as_with::<_, UserRow, _>(&sql, values)
            .fetch_optional(&self.db)
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

        let access_token = jwt::generate_access_token(user.id, &user.role)
            .map_err(|_| AuthServiceError::Internal("token generation failed".into()))?;
        let refresh_token = jwt::generate_refresh_token(user.id)
            .map_err(|_| AuthServiceError::Internal("token generation failed".into()))?;

        let token_hash = hash_token(&refresh_token);
        let now = chrono::Utc::now().naive_utc();
        let expires_at =
            now + chrono::Duration::seconds(self.config.refresh_token_ttl.cast_signed());

        let (sql, values) = Query::insert()
            .into_table(RefreshToken::Table)
            .columns([
                RefreshToken::UserId,
                RefreshToken::TokenHash,
                RefreshToken::ExpiresAt,
                RefreshToken::CreatedAt,
            ])
            .values_panic([
                user.id.into(),
                token_hash.into(),
                expires_at.to_string().into(),
                now.to_string().into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        Ok(AuthResponse {
            user: UserResponse::from(user),
            access_token,
            refresh_token,
        })
    }

    async fn verify_email(&self, raw_token: &str) -> Result<VerifyEmailResponse, AuthServiceError> {
        use entity::email_verification_token::{EmailVerificationToken, EmailVerificationTokenRow};
        use entity::user::{User, UserRow};

        let mut txn = self.db.begin().await.map_err(AuthServiceError::Database)?;

        let token_hash = hash_token(raw_token);

        // Find verification token
        let (sql, values) = Query::select()
            .columns([
                EmailVerificationToken::Id,
                EmailVerificationToken::UserId,
                EmailVerificationToken::TokenHash,
                EmailVerificationToken::ExpiresAt,
                EmailVerificationToken::CreatedAt,
            ])
            .from(EmailVerificationToken::Table)
            .and_where(Expr::col(EmailVerificationToken::TokenHash).eq(&token_hash))
            .build_sqlx(SqliteQueryBuilder);

        let stored = sqlx::query_as_with::<_, EmailVerificationTokenRow, _>(&sql, values)
            .fetch_optional(&mut *txn)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or_else(|| AuthServiceError::NotFound("Invalid verification token".to_string()))?;

        let now = chrono::Utc::now().fixed_offset();
        if stored.expires_at < now {
            // Delete expired token
            let (sql, values) = Query::delete()
                .from_table(EmailVerificationToken::Table)
                .and_where(Expr::col(EmailVerificationToken::Id).eq(stored.id))
                .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *txn)
                .await
                .map_err(AuthServiceError::Database)?;

            return Err(AuthServiceError::NotFound(
                "Verification token expired. Please request a new one.".to_string(),
            ));
        }

        // Find user
        let (sql, values) = Query::select()
            .columns([
                User::Id,
                User::Email,
                User::PasswordHash,
                User::Role,
                User::IsActive,
                User::EmailVerifiedAt,
                User::CreatedAt,
                User::UpdatedAt,
            ])
            .from(User::Table)
            .and_where(Expr::col(User::Id).eq(stored.user_id))
            .build_sqlx(SqliteQueryBuilder);

        let _user = sqlx::query_as_with::<_, UserRow, _>(&sql, values)
            .fetch_optional(&mut *txn)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or_else(|| AuthServiceError::NotFound("User not found".to_string()))?;

        // Update user email_verified_at
        let (sql, values) = Query::update()
            .table(User::Table)
            .values([(User::EmailVerifiedAt, now.to_rfc3339().into())])
            .and_where(Expr::col(User::Id).eq(stored.user_id))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *txn)
            .await
            .map_err(AuthServiceError::Database)?;

        // Delete verification token
        let (sql, values) = Query::delete()
            .from_table(EmailVerificationToken::Table)
            .and_where(Expr::col(EmailVerificationToken::Id).eq(stored.id))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *txn)
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
        use entity::email_verification_token::EmailVerificationToken;
        use entity::user::{User, UserRow};

        let generic_message = ResendVerificationResponse {
            message: "If an account with that email exists and is not yet verified, we've sent a verification email.".to_string(),
        };

        // Find user
        let (sql, values) = Query::select()
            .columns([
                User::Id,
                User::Email,
                User::PasswordHash,
                User::Role,
                User::IsActive,
                User::EmailVerifiedAt,
                User::CreatedAt,
                User::UpdatedAt,
            ])
            .from(User::Table)
            .and_where(Expr::col(User::Email).eq(email))
            .build_sqlx(SqliteQueryBuilder);

        let user = sqlx::query_as_with::<_, UserRow, _>(&sql, values)
            .fetch_optional(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        let Some(user) = user else {
            return Ok(generic_message);
        };

        if user.email_verified_at.is_some() {
            return Ok(generic_message);
        }

        // Delete existing verification tokens
        let (sql, values) = Query::delete()
            .from_table(EmailVerificationToken::Table)
            .and_where(Expr::col(EmailVerificationToken::UserId).eq(user.id))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        // Generate new verification token
        let raw_token = uuid::Uuid::new_v4().to_string();
        let token_hash = hash_token(&raw_token);
        let now_with_tz = chrono::Utc::now().fixed_offset();
        let expires_at = now_with_tz
            + chrono::Duration::seconds(self.config.email_verification_token_ttl.cast_signed());

        let (sql, values) = Query::insert()
            .into_table(EmailVerificationToken::Table)
            .columns([
                EmailVerificationToken::UserId,
                EmailVerificationToken::TokenHash,
                EmailVerificationToken::ExpiresAt,
                EmailVerificationToken::CreatedAt,
            ])
            .values_panic([
                user.id.into(),
                token_hash.into(),
                expires_at.to_rfc3339().into(),
                now_with_tz.to_rfc3339().into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
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
        use entity::refresh_token::{RefreshToken, RefreshTokenRow};
        use entity::user::{User, UserRow};

        let claims = jwt::validate_refresh_token(token)
            .map_err(|_| AuthServiceError::Unauthorized("Invalid refresh token".to_string()))?;

        let token_hash = hash_token(token);
        let now = chrono::Utc::now().naive_utc();

        // Find refresh token
        let (sql, values) = Query::select()
            .columns([
                RefreshToken::Id,
                RefreshToken::UserId,
                RefreshToken::TokenHash,
                RefreshToken::ExpiresAt,
                RefreshToken::RevokedAt,
                RefreshToken::CreatedAt,
            ])
            .from(RefreshToken::Table)
            .and_where(Expr::col(RefreshToken::TokenHash).eq(&token_hash))
            .build_sqlx(SqliteQueryBuilder);

        let stored = sqlx::query_as_with::<_, RefreshTokenRow, _>(&sql, values)
            .fetch_optional(&self.db)
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

        // Revoke old token
        let (sql, values) = Query::update()
            .table(RefreshToken::Table)
            .values([(RefreshToken::RevokedAt, now.to_string().into())])
            .and_where(Expr::col(RefreshToken::Id).eq(stored.id))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        // Find user
        let user_id: i32 = claims
            .sub
            .parse()
            .map_err(|_| AuthServiceError::Unauthorized("Invalid token subject".to_string()))?;

        let (sql, values) = Query::select()
            .columns([
                User::Id,
                User::Email,
                User::PasswordHash,
                User::Role,
                User::IsActive,
                User::EmailVerifiedAt,
                User::CreatedAt,
                User::UpdatedAt,
            ])
            .from(User::Table)
            .and_where(Expr::col(User::Id).eq(user_id))
            .build_sqlx(SqliteQueryBuilder);

        let user = sqlx::query_as_with::<_, UserRow, _>(&sql, values)
            .fetch_optional(&self.db)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or_else(|| AuthServiceError::NotFound("User not found".to_string()))?;

        if !user.is_active {
            return Err(AuthServiceError::Unauthorized(
                "Account is deactivated".to_string(),
            ));
        }

        let access_token = jwt::generate_access_token(user.id, &user.role)
            .map_err(|_| AuthServiceError::Internal("token generation failed".into()))?;
        let new_refresh_token = jwt::generate_refresh_token(user.id)
            .map_err(|_| AuthServiceError::Internal("token generation failed".into()))?;

        let new_token_hash = hash_token(&new_refresh_token);
        let expires_at =
            now + chrono::Duration::seconds(self.config.refresh_token_ttl.cast_signed());

        let (sql, values) = Query::insert()
            .into_table(RefreshToken::Table)
            .columns([
                RefreshToken::UserId,
                RefreshToken::TokenHash,
                RefreshToken::ExpiresAt,
                RefreshToken::CreatedAt,
            ])
            .values_panic([
                user.id.into(),
                new_token_hash.into(),
                expires_at.to_string().into(),
                now.to_string().into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        Ok(RefreshResponse {
            access_token,
            refresh_token: new_refresh_token,
        })
    }

    async fn logout(&self, token: &str) -> Result<(), AuthServiceError> {
        use entity::refresh_token::{RefreshToken, RefreshTokenRow};

        let _claims = jwt::validate_refresh_token(token)
            .map_err(|_| AuthServiceError::Unauthorized("Invalid refresh token".to_string()))?;

        let token_hash = hash_token(token);
        let now = chrono::Utc::now().naive_utc();

        // Find refresh token
        let (sql, values) = Query::select()
            .columns([
                RefreshToken::Id,
                RefreshToken::UserId,
                RefreshToken::TokenHash,
                RefreshToken::ExpiresAt,
                RefreshToken::RevokedAt,
                RefreshToken::CreatedAt,
            ])
            .from(RefreshToken::Table)
            .and_where(Expr::col(RefreshToken::TokenHash).eq(&token_hash))
            .build_sqlx(SqliteQueryBuilder);

        let stored = sqlx::query_as_with::<_, RefreshTokenRow, _>(&sql, values)
            .fetch_optional(&self.db)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or_else(|| AuthServiceError::Unauthorized("Invalid refresh token".to_string()))?;

        if stored.revoked_at.is_some() {
            return Ok(());
        }

        // Revoke token
        let (sql, values) = Query::update()
            .table(RefreshToken::Table)
            .values([(RefreshToken::RevokedAt, now.to_string().into())])
            .and_where(Expr::col(RefreshToken::Id).eq(stored.id))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        Ok(())
    }

    async fn get_me(&self, user_id: i32) -> Result<MeResponse, AuthServiceError> {
        use entity::user::{User, UserRow};

        let (sql, values) = Query::select()
            .columns([
                User::Id,
                User::Email,
                User::PasswordHash,
                User::Role,
                User::IsActive,
                User::EmailVerifiedAt,
                User::CreatedAt,
                User::UpdatedAt,
            ])
            .from(User::Table)
            .and_where(Expr::col(User::Id).eq(user_id))
            .build_sqlx(SqliteQueryBuilder);

        let user = sqlx::query_as_with::<_, UserRow, _>(&sql, values)
            .fetch_optional(&self.db)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or_else(|| AuthServiceError::Unauthorized("User not found".to_string()))?;

        Ok(MeResponse::from(user))
    }
}
