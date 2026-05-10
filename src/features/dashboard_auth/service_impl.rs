use std::sync::Arc;

use async_trait::async_trait;
use bcrypt::{hash, verify};
use rand::RngCore;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::email_service::EmailService;
use super::service::{
    DashboardAuthResponse, DashboardAuthService, DashboardAuthServiceError, DashboardMeResponse,
    DashboardRefreshResponse, DashboardRegisterResponse, DashboardResendVerificationResponse,
    DashboardVerifyEmailResponse, EmployeeResponse,
};
use crate::auth::jwt;
use crate::config::Config;

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn generate_secure_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

#[derive(Clone, Debug)]
pub struct DashboardAuthServiceImpl {
    pub db: SqlitePool,
    pub email_service: Arc<dyn EmailService>,
    pub config: Arc<Config>,
}

#[async_trait]
impl DashboardAuthService for DashboardAuthServiceImpl {
    async fn register(
        &self,
        email: &str,
        password: &str,
    ) -> Result<DashboardRegisterResponse, DashboardAuthServiceError> {
        use entity::employee::Employee;
        use entity::employee_email_verification_token::EmployeeEmailVerificationToken;

        if password.len() < 8 {
            return Err(DashboardAuthServiceError::BadRequest(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        let mut txn = self
            .db
            .begin()
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        let (sql, values) = Query::select()
            .columns([Employee::Id])
            .from(Employee::Table)
            .build_sqlx(SqliteQueryBuilder);

        let existing: Option<(Uuid,)> = sqlx::query_as_with(&sql, values)
            .fetch_optional(&mut *txn)
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        if existing.is_some() {
            return Err(DashboardAuthServiceError::Conflict(
                "An owner account already exists".to_string(),
            ));
        }

        let (sql, values) = Query::select()
            .columns([Employee::Id])
            .from(Employee::Table)
            .and_where(Expr::col(Employee::Email).eq(email))
            .build_sqlx(SqliteQueryBuilder);

        let existing_email: Option<(Uuid,)> = sqlx::query_as_with(&sql, values)
            .fetch_optional(&mut *txn)
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        if existing_email.is_some() {
            return Err(DashboardAuthServiceError::Conflict(
                "Email already registered".to_string(),
            ));
        }

        let password_hash = hash(password, self.config.bcrypt_cost)
            .map_err(|_| DashboardAuthServiceError::Internal("password hashing failed".into()))?;

        let employee_id = Uuid::now_v7();
        let now = chrono::Utc::now();
        let role = "owner";

        let (sql, values) = Query::insert()
            .into_table(Employee::Table)
            .columns([
                Employee::Id,
                Employee::Email,
                Employee::PasswordHash,
                Employee::Role,
                Employee::IsActive,
                Employee::UpdatedAt,
                Employee::CreatedAt,
            ])
            .values_panic([
                employee_id.into(),
                email.into(),
                password_hash.into(),
                role.into(),
                true.into(),
                now.to_rfc3339().into(),
                now.to_rfc3339().into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *txn)
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        let raw_token = generate_secure_token();
        let token_hash = hash_token(&raw_token);
        let expires_at =
            now + chrono::Duration::seconds(self.config.email_verification_token_ttl.cast_signed());

        let (sql, values) = Query::insert()
            .into_table(EmployeeEmailVerificationToken::Table)
            .columns([
                EmployeeEmailVerificationToken::Id,
                EmployeeEmailVerificationToken::EmployeeId,
                EmployeeEmailVerificationToken::TokenHash,
                EmployeeEmailVerificationToken::ExpiresAt,
                EmployeeEmailVerificationToken::CreatedAt,
            ])
            .values_panic([
                Uuid::now_v7().into(),
                employee_id.into(),
                token_hash.into(),
                expires_at.to_rfc3339().into(),
                now.to_rfc3339().into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *txn)
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        txn.commit()
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        let verification_link = format!(
            "{}/api/v1/dashboard/auth/verify-email?token={}",
            self.config.base_url, raw_token
        );

        if let Err(e) = self
            .email_service
            .send_verification_email(email, &verification_link)
            .await
        {
            tracing::warn!(error = %e, "failed to send verification email");
        }

        let _ = self.cleanup_expired_tokens().await;

        Ok(DashboardRegisterResponse {
            message: "Please check your email to verify your account.".to_string(),
        })
    }

    async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<DashboardAuthResponse, DashboardAuthServiceError> {
        use entity::employee::Employee;
        use entity::employee::EmployeeRow;
        use entity::employee_refresh_token::EmployeeRefreshToken;

        let (sql, values) = Query::select()
            .columns([
                Employee::Id,
                Employee::Email,
                Employee::PasswordHash,
                Employee::Role,
                Employee::IsActive,
                Employee::EmailVerifiedAt,
                Employee::CreatedAt,
                Employee::UpdatedAt,
            ])
            .from(Employee::Table)
            .and_where(Expr::col(Employee::Email).eq(email))
            .build_sqlx(SqliteQueryBuilder);

        let employee = sqlx::query_as_with::<_, EmployeeRow, _>(&sql, values)
            .fetch_optional(&self.db)
            .await
            .map_err(DashboardAuthServiceError::Database)?
            .ok_or_else(|| {
                DashboardAuthServiceError::Unauthorized("Invalid email or password".to_string())
            })?;

        if employee.email_verified_at.is_none() {
            return Err(DashboardAuthServiceError::NotVerified(
                "Email not verified. Please check your email.".to_string(),
            ));
        }

        if !employee.is_active {
            return Err(DashboardAuthServiceError::Unauthorized(
                "Account is deactivated".to_string(),
            ));
        }

        let valid = verify(password, &employee.password_hash).map_err(|_| {
            DashboardAuthServiceError::Unauthorized("Invalid email or password".to_string())
        })?;
        if !valid {
            return Err(DashboardAuthServiceError::Unauthorized(
                "Invalid email or password".to_string(),
            ));
        }

        let access_token = jwt::generate_access_token(employee.id, &employee.role)
            .map_err(|_| DashboardAuthServiceError::Internal("token generation failed".into()))?;
        let refresh_token = jwt::generate_refresh_token(employee.id)
            .map_err(|_| DashboardAuthServiceError::Internal("token generation failed".into()))?;

        let token_hash = hash_token(&refresh_token);
        let now = chrono::Utc::now();
        let expires_at =
            now + chrono::Duration::seconds(self.config.refresh_token_ttl.cast_signed());

        let (sql, values) = Query::insert()
            .into_table(EmployeeRefreshToken::Table)
            .columns([
                EmployeeRefreshToken::Id,
                EmployeeRefreshToken::EmployeeId,
                EmployeeRefreshToken::TokenHash,
                EmployeeRefreshToken::ExpiresAt,
                EmployeeRefreshToken::CreatedAt,
            ])
            .values_panic([
                Uuid::now_v7().into(),
                employee.id.into(),
                token_hash.into(),
                expires_at.to_rfc3339().into(),
                now.to_rfc3339().into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        let _ = self.cleanup_expired_tokens().await;

        Ok(DashboardAuthResponse {
            employee: EmployeeResponse::from(employee),
            access_token,
            refresh_token,
        })
    }

    async fn verify_email(
        &self,
        raw_token: &str,
    ) -> Result<DashboardVerifyEmailResponse, DashboardAuthServiceError> {
        use entity::employee::Employee;
        use entity::employee_email_verification_token::{
            EmployeeEmailVerificationToken, EmployeeEmailVerificationTokenRow,
        };

        let mut txn = self
            .db
            .begin()
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        let token_hash = hash_token(raw_token);

        let (sql, values) = Query::select()
            .columns([
                EmployeeEmailVerificationToken::Id,
                EmployeeEmailVerificationToken::EmployeeId,
                EmployeeEmailVerificationToken::TokenHash,
                EmployeeEmailVerificationToken::ExpiresAt,
                EmployeeEmailVerificationToken::CreatedAt,
            ])
            .from(EmployeeEmailVerificationToken::Table)
            .and_where(Expr::col(EmployeeEmailVerificationToken::TokenHash).eq(&token_hash))
            .build_sqlx(SqliteQueryBuilder);

        let stored = sqlx::query_as_with::<_, EmployeeEmailVerificationTokenRow, _>(&sql, values)
            .fetch_optional(&mut *txn)
            .await
            .map_err(DashboardAuthServiceError::Database)?
            .ok_or_else(|| {
                DashboardAuthServiceError::NotFound("Invalid verification token".to_string())
            })?;

        let now = chrono::Utc::now();
        if stored.expires_at < now {
            let (sql, values) = Query::delete()
                .from_table(EmployeeEmailVerificationToken::Table)
                .and_where(Expr::col(EmployeeEmailVerificationToken::Id).eq(stored.id))
                .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *txn)
                .await
                .map_err(DashboardAuthServiceError::Database)?;

            return Err(DashboardAuthServiceError::NotFound(
                "Verification token expired. Please request a new one.".to_string(),
            ));
        }

        let (sql, values) = Query::update()
            .table(Employee::Table)
            .values([(Employee::EmailVerifiedAt, now.to_rfc3339().into())])
            .and_where(Expr::col(Employee::Id).eq(stored.employee_id))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *txn)
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        let (sql, values) = Query::delete()
            .from_table(EmployeeEmailVerificationToken::Table)
            .and_where(Expr::col(EmployeeEmailVerificationToken::Id).eq(stored.id))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *txn)
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        txn.commit()
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        Ok(DashboardVerifyEmailResponse {
            message: "Email verified successfully. You can now log in.".to_string(),
        })
    }

    async fn resend_verification(
        &self,
        email: &str,
    ) -> Result<DashboardResendVerificationResponse, DashboardAuthServiceError> {
        use entity::employee::Employee;
        use entity::employee::EmployeeRow;
        use entity::employee_email_verification_token::EmployeeEmailVerificationToken;

        let generic_message = DashboardResendVerificationResponse {
            message: "If an account with that email exists and is not yet verified, we've sent a verification email.".to_string(),
        };

        let (sql, values) = Query::select()
            .columns([
                Employee::Id,
                Employee::Email,
                Employee::PasswordHash,
                Employee::Role,
                Employee::IsActive,
                Employee::EmailVerifiedAt,
                Employee::CreatedAt,
                Employee::UpdatedAt,
            ])
            .from(Employee::Table)
            .and_where(Expr::col(Employee::Email).eq(email))
            .build_sqlx(SqliteQueryBuilder);

        let employee = sqlx::query_as_with::<_, EmployeeRow, _>(&sql, values)
            .fetch_optional(&self.db)
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        let Some(employee) = employee else {
            return Ok(generic_message);
        };

        if employee.email_verified_at.is_some() {
            return Ok(generic_message);
        }

        let (sql, values) = Query::delete()
            .from_table(EmployeeEmailVerificationToken::Table)
            .and_where(Expr::col(EmployeeEmailVerificationToken::EmployeeId).eq(employee.id))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        let raw_token = generate_secure_token();
        let token_hash = hash_token(&raw_token);
        let now = chrono::Utc::now();
        let expires_at =
            now + chrono::Duration::seconds(self.config.email_verification_token_ttl.cast_signed());

        let (sql, values) = Query::insert()
            .into_table(EmployeeEmailVerificationToken::Table)
            .columns([
                EmployeeEmailVerificationToken::Id,
                EmployeeEmailVerificationToken::EmployeeId,
                EmployeeEmailVerificationToken::TokenHash,
                EmployeeEmailVerificationToken::ExpiresAt,
                EmployeeEmailVerificationToken::CreatedAt,
            ])
            .values_panic([
                Uuid::now_v7().into(),
                employee.id.into(),
                token_hash.into(),
                expires_at.to_rfc3339().into(),
                now.to_rfc3339().into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        let verification_link = format!(
            "{}/api/v1/dashboard/auth/verify-email?token={}",
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

    async fn refresh(
        &self,
        token: &str,
    ) -> Result<DashboardRefreshResponse, DashboardAuthServiceError> {
        use entity::employee::Employee;
        use entity::employee::EmployeeRow;
        use entity::employee_refresh_token::{EmployeeRefreshToken, EmployeeRefreshTokenRow};

        let claims = jwt::validate_refresh_token(token).map_err(|_| {
            DashboardAuthServiceError::Unauthorized("Invalid refresh token".to_string())
        })?;

        let token_hash = hash_token(token);

        let mut txn = self
            .db
            .begin()
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        let (sql, values) = Query::select()
            .columns([
                EmployeeRefreshToken::Id,
                EmployeeRefreshToken::EmployeeId,
                EmployeeRefreshToken::TokenHash,
                EmployeeRefreshToken::ExpiresAt,
                EmployeeRefreshToken::RevokedAt,
                EmployeeRefreshToken::CreatedAt,
            ])
            .from(EmployeeRefreshToken::Table)
            .and_where(Expr::col(EmployeeRefreshToken::TokenHash).eq(&token_hash))
            .build_sqlx(SqliteQueryBuilder);

        let stored = sqlx::query_as_with::<_, EmployeeRefreshTokenRow, _>(&sql, values)
            .fetch_optional(&mut *txn)
            .await
            .map_err(DashboardAuthServiceError::Database)?
            .ok_or_else(|| {
                DashboardAuthServiceError::Unauthorized("Invalid refresh token".to_string())
            })?;

        if stored.revoked_at.is_some() {
            return Err(DashboardAuthServiceError::Unauthorized(
                "Refresh token revoked".to_string(),
            ));
        }

        let now = chrono::Utc::now();
        if stored.expires_at < now {
            return Err(DashboardAuthServiceError::Unauthorized(
                "Refresh token expired".to_string(),
            ));
        }

        let (sql, values) = Query::update()
            .table(EmployeeRefreshToken::Table)
            .values([(EmployeeRefreshToken::RevokedAt, now.to_rfc3339().into())])
            .and_where(Expr::col(EmployeeRefreshToken::Id).eq(stored.id))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *txn)
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        let employee_id: Uuid = claims.sub.parse().map_err(|_| {
            DashboardAuthServiceError::Unauthorized("Invalid token subject".to_string())
        })?;

        let (sql, values) = Query::select()
            .columns([
                Employee::Id,
                Employee::Email,
                Employee::PasswordHash,
                Employee::Role,
                Employee::IsActive,
                Employee::EmailVerifiedAt,
                Employee::CreatedAt,
                Employee::UpdatedAt,
            ])
            .from(Employee::Table)
            .and_where(Expr::col(Employee::Id).eq(employee_id))
            .build_sqlx(SqliteQueryBuilder);

        let employee = sqlx::query_as_with::<_, EmployeeRow, _>(&sql, values)
            .fetch_optional(&mut *txn)
            .await
            .map_err(DashboardAuthServiceError::Database)?
            .ok_or_else(|| DashboardAuthServiceError::NotFound("Employee not found".to_string()))?;

        if !employee.is_active {
            return Err(DashboardAuthServiceError::Unauthorized(
                "Account is deactivated".to_string(),
            ));
        }

        let access_token = jwt::generate_access_token(employee.id, &employee.role)
            .map_err(|_| DashboardAuthServiceError::Internal("token generation failed".into()))?;
        let new_refresh_token = jwt::generate_refresh_token(employee.id)
            .map_err(|_| DashboardAuthServiceError::Internal("token generation failed".into()))?;

        let new_token_hash = hash_token(&new_refresh_token);
        let expires_at =
            now + chrono::Duration::seconds(self.config.refresh_token_ttl.cast_signed());

        let (sql, values) = Query::insert()
            .into_table(EmployeeRefreshToken::Table)
            .columns([
                EmployeeRefreshToken::Id,
                EmployeeRefreshToken::EmployeeId,
                EmployeeRefreshToken::TokenHash,
                EmployeeRefreshToken::ExpiresAt,
                EmployeeRefreshToken::CreatedAt,
            ])
            .values_panic([
                Uuid::now_v7().into(),
                employee.id.into(),
                new_token_hash.into(),
                expires_at.to_rfc3339().into(),
                now.to_rfc3339().into(),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&mut *txn)
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        txn.commit()
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        Ok(DashboardRefreshResponse {
            access_token,
            refresh_token: new_refresh_token,
        })
    }

    async fn logout(&self, token: &str) -> Result<(), DashboardAuthServiceError> {
        use entity::employee_refresh_token::{EmployeeRefreshToken, EmployeeRefreshTokenRow};

        let _claims = jwt::validate_refresh_token(token).map_err(|_| {
            DashboardAuthServiceError::Unauthorized("Invalid refresh token".to_string())
        })?;

        let token_hash = hash_token(token);
        let now = chrono::Utc::now();

        let (sql, values) = Query::select()
            .columns([
                EmployeeRefreshToken::Id,
                EmployeeRefreshToken::EmployeeId,
                EmployeeRefreshToken::TokenHash,
                EmployeeRefreshToken::ExpiresAt,
                EmployeeRefreshToken::RevokedAt,
                EmployeeRefreshToken::CreatedAt,
            ])
            .from(EmployeeRefreshToken::Table)
            .and_where(Expr::col(EmployeeRefreshToken::TokenHash).eq(&token_hash))
            .build_sqlx(SqliteQueryBuilder);

        let stored = sqlx::query_as_with::<_, EmployeeRefreshTokenRow, _>(&sql, values)
            .fetch_optional(&self.db)
            .await
            .map_err(DashboardAuthServiceError::Database)?
            .ok_or_else(|| {
                DashboardAuthServiceError::Unauthorized("Invalid refresh token".to_string())
            })?;

        if stored.revoked_at.is_some() {
            return Ok(());
        }

        let (sql, values) = Query::update()
            .table(EmployeeRefreshToken::Table)
            .values([(EmployeeRefreshToken::RevokedAt, now.to_rfc3339().into())])
            .and_where(Expr::col(EmployeeRefreshToken::Id).eq(stored.id))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(DashboardAuthServiceError::Database)?;

        Ok(())
    }

    async fn get_me(
        &self,
        employee_id: Uuid,
    ) -> Result<DashboardMeResponse, DashboardAuthServiceError> {
        use entity::employee::Employee;
        use entity::employee::EmployeeRow;

        let (sql, values) = Query::select()
            .columns([
                Employee::Id,
                Employee::Email,
                Employee::PasswordHash,
                Employee::Role,
                Employee::IsActive,
                Employee::EmailVerifiedAt,
                Employee::CreatedAt,
                Employee::UpdatedAt,
            ])
            .from(Employee::Table)
            .and_where(Expr::col(Employee::Id).eq(employee_id))
            .build_sqlx(SqliteQueryBuilder);

        let employee = sqlx::query_as_with::<_, EmployeeRow, _>(&sql, values)
            .fetch_optional(&self.db)
            .await
            .map_err(DashboardAuthServiceError::Database)?
            .ok_or_else(|| DashboardAuthServiceError::NotFound("Employee not found".to_string()))?;

        Ok(DashboardMeResponse::from(employee))
    }
}

impl DashboardAuthServiceImpl {
    async fn cleanup_expired_tokens(&self) -> Result<(), sqlx::Error> {
        use entity::employee_email_verification_token::EmployeeEmailVerificationToken;
        use entity::employee_refresh_token::EmployeeRefreshToken;

        let now = chrono::Utc::now();

        let (sql, values) = Query::delete()
            .from_table(EmployeeEmailVerificationToken::Table)
            .and_where(Expr::col(EmployeeEmailVerificationToken::ExpiresAt).lt(now.to_rfc3339()))
            .build_sqlx(SqliteQueryBuilder);

        let _ = sqlx::query_with(&sql, values).execute(&self.db).await;

        let (sql, values) = Query::delete()
            .from_table(EmployeeRefreshToken::Table)
            .and_where(Expr::col(EmployeeRefreshToken::ExpiresAt).lt(now.to_rfc3339()))
            .build_sqlx(SqliteQueryBuilder);

        let _ = sqlx::query_with(&sql, values).execute(&self.db).await;

        Ok(())
    }
}
