use async_trait::async_trait;
use bcrypt::{DEFAULT_COST, hash, verify};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use sha2::{Digest, Sha256};

use super::service::{AuthResponse, AuthService, AuthServiceError, RefreshResponse, UserResponse};
use crate::auth::jwt;

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(Clone, Debug)]
pub struct AuthServiceImpl {
    pub db: DatabaseConnection,
}

#[async_trait]
impl AuthService for AuthServiceImpl {
    async fn register(
        &self,
        username: String,
        email: String,
        password: String,
        role: String,
    ) -> Result<AuthResponse, AuthServiceError> {
        let existing = entity::user::Entity::find()
            .filter(entity::user::Column::Username.eq(&username))
            .one(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        if existing.is_some() {
            return Err(AuthServiceError::Conflict(format!(
                "username '{username}' already exists"
            )));
        }

        let password_hash = hash(&password, DEFAULT_COST).map_err(|_| {
            AuthServiceError::Database(sea_orm::DbErr::Custom("password hashing failed".into()))
        })?;

        let now = chrono::Utc::now().naive_utc();

        let user_model = entity::user::ActiveModel {
            username: Set(username),
            email: Set(email),
            password_hash: Set(password_hash),
            role: Set(role),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let user = user_model
            .insert(&self.db)
            .await
            .map_err(AuthServiceError::Database)?;

        let access_token = jwt::generate_access_token(user.id, &user.role).map_err(|_| {
            AuthServiceError::Database(sea_orm::DbErr::Custom("token generation failed".into()))
        })?;
        let refresh_token = jwt::generate_refresh_token(user.id).map_err(|_| {
            AuthServiceError::Database(sea_orm::DbErr::Custom("token generation failed".into()))
        })?;

        let token_hash = hash_token(&refresh_token);
        let config = crate::config::get_config();
        let expires_at = chrono::Utc::now().naive_utc()
            + chrono::Duration::seconds(config.refresh_token_ttl as i64);

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

    async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<AuthResponse, AuthServiceError> {
        let user = entity::user::Entity::find()
            .filter(entity::user::Column::Username.eq(&username))
            .one(&self.db)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or(AuthServiceError::NotFound)?;

        let valid =
            verify(&password, &user.password_hash).map_err(|_| AuthServiceError::Unauthorized)?;
        if !valid {
            return Err(AuthServiceError::Unauthorized);
        }

        if !user.is_active {
            return Err(AuthServiceError::Unauthorized);
        }

        let access_token = jwt::generate_access_token(user.id, &user.role).map_err(|_| {
            AuthServiceError::Database(sea_orm::DbErr::Custom("token generation failed".into()))
        })?;
        let refresh_token = jwt::generate_refresh_token(user.id).map_err(|_| {
            AuthServiceError::Database(sea_orm::DbErr::Custom("token generation failed".into()))
        })?;

        let token_hash = hash_token(&refresh_token);
        let now = chrono::Utc::now().naive_utc();
        let config = crate::config::get_config();
        let expires_at = now + chrono::Duration::seconds(config.refresh_token_ttl as i64);

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

    async fn refresh(&self, token: String) -> Result<RefreshResponse, AuthServiceError> {
        let claims =
            jwt::validate_refresh_token(&token).map_err(|_| AuthServiceError::Unauthorized)?;

        let token_hash = hash_token(&token);
        let now = chrono::Utc::now().naive_utc();

        let stored = entity::refresh_token::Entity::find()
            .filter(entity::refresh_token::Column::TokenHash.eq(&token_hash))
            .one(&self.db)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or(AuthServiceError::Unauthorized)?;

        if stored.revoked_at.is_some() {
            return Err(AuthServiceError::Unauthorized);
        }

        if stored.expires_at < now {
            return Err(AuthServiceError::Unauthorized);
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
            .map_err(|_| AuthServiceError::Unauthorized)?;
        let user = entity::user::Entity::find_by_id(user_id)
            .one(&self.db)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or(AuthServiceError::NotFound)?;

        if !user.is_active {
            return Err(AuthServiceError::Unauthorized);
        }

        let access_token = jwt::generate_access_token(user.id, &user.role).map_err(|_| {
            AuthServiceError::Database(sea_orm::DbErr::Custom("token generation failed".into()))
        })?;
        let new_refresh_token = jwt::generate_refresh_token(user.id).map_err(|_| {
            AuthServiceError::Database(sea_orm::DbErr::Custom("token generation failed".into()))
        })?;

        let new_token_hash = hash_token(&new_refresh_token);
        let config = crate::config::get_config();
        let expires_at = now + chrono::Duration::seconds(config.refresh_token_ttl as i64);

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

    async fn logout(&self, token: String) -> Result<(), AuthServiceError> {
        let _claims =
            jwt::validate_refresh_token(&token).map_err(|_| AuthServiceError::Unauthorized)?;

        let token_hash = hash_token(&token);
        let now = chrono::Utc::now().naive_utc();

        let stored = entity::refresh_token::Entity::find()
            .filter(entity::refresh_token::Column::TokenHash.eq(&token_hash))
            .one(&self.db)
            .await
            .map_err(AuthServiceError::Database)?
            .ok_or(AuthServiceError::Unauthorized)?;

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
