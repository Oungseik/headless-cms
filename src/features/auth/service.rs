use std::fmt;

use async_trait::async_trait;
use chrono::NaiveDateTime;

#[derive(Debug)]
pub enum AuthServiceError {
    NotFound,
    Unauthorized,
    Conflict(String),
    Database(sea_orm::DbErr),
}

impl fmt::Display for AuthServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "user not found"),
            Self::Unauthorized => write!(f, "invalid credentials"),
            Self::Conflict(msg) => write!(f, "conflict: {msg}"),
            Self::Database(err) => write!(f, "database error: {err}"),
        }
    }
}

impl std::error::Error for AuthServiceError {}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
}

impl From<entity::user::Model> for UserResponse {
    fn from(m: entity::user::Model) -> Self {
        Self {
            id: m.id,
            username: m.username,
            email: m.email,
            role: m.role,
            is_active: m.is_active,
            created_at: m.created_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[async_trait]
pub trait AuthService: Send + Sync + 'static {
    async fn register(
        &self,
        username: String,
        email: String,
        password: String,
        role: String,
    ) -> Result<AuthResponse, AuthServiceError>;
    async fn login(
        &self,
        username: String,
        password: String,
    ) -> Result<AuthResponse, AuthServiceError>;
    async fn refresh(&self, token: String) -> Result<RefreshResponse, AuthServiceError>;
    async fn logout(&self, token: String) -> Result<(), AuthServiceError>;
}
