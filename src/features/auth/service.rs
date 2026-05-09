use std::fmt;

use async_trait::async_trait;
use chrono::FixedOffset;
use chrono::{DateTime, NaiveDateTime};

#[derive(Debug)]
pub enum AuthServiceError {
    NotFound(String),
    Unauthorized(String),
    NotVerified(String),
    Conflict(String),
    Database(sea_orm::DbErr),
}

impl fmt::Display for AuthServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
            Self::Unauthorized(msg) => write!(f, "unauthorized: {msg}"),
            Self::NotVerified(msg) => write!(f, "not verified: {msg}"),
            Self::Conflict(msg) => write!(f, "conflict: {msg}"),
            Self::Database(err) => write!(f, "database error: {err}"),
        }
    }
}

impl std::error::Error for AuthServiceError {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: i32,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
}

impl From<entity::user::Model> for UserResponse {
    fn from(m: entity::user::Model) -> Self {
        Self {
            id: m.id,
            email: m.email,
            role: m.role,
            is_active: m.is_active,
            created_at: m.created_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyEmailResponse {
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResendVerificationResponse {
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MeResponse {
    pub id: i32,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub email_verified_at: Option<DateTime<FixedOffset>>,
    pub updated_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

impl From<entity::user::Model> for MeResponse {
    fn from(m: entity::user::Model) -> Self {
        Self {
            id: m.id,
            email: m.email,
            role: m.role,
            is_active: m.is_active,
            email_verified_at: m.email_verified_at,
            updated_at: m.updated_at,
            created_at: m.created_at,
        }
    }
}

#[async_trait]
pub trait AuthService: Send + Sync + 'static {
    async fn register(
        &self,
        email: &str,
        password: &str,
        role: &str,
    ) -> Result<RegisterResponse, AuthServiceError>;
    async fn login(&self, email: &str, password: &str) -> Result<AuthResponse, AuthServiceError>;
    async fn verify_email(&self, token: &str) -> Result<VerifyEmailResponse, AuthServiceError>;
    async fn resend_verification(
        &self,
        email: &str,
    ) -> Result<ResendVerificationResponse, AuthServiceError>;
    async fn refresh(&self, token: &str) -> Result<RefreshResponse, AuthServiceError>;
    async fn logout(&self, token: &str) -> Result<(), AuthServiceError>;
    async fn get_me(&self, user_id: i32) -> Result<MeResponse, AuthServiceError>;
}
