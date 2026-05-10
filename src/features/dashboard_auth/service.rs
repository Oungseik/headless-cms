use std::fmt;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub enum DashboardAuthServiceError {
    NotFound(String),
    Unauthorized(String),
    NotVerified(String),
    Conflict(String),
    BadRequest(String),
    Internal(String),
    Database(sqlx::Error),
}

impl fmt::Display for DashboardAuthServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
            Self::Unauthorized(msg) => write!(f, "unauthorized: {msg}"),
            Self::NotVerified(msg) => write!(f, "not verified: {msg}"),
            Self::Conflict(msg) => write!(f, "conflict: {msg}"),
            Self::BadRequest(msg) => write!(f, "bad request: {msg}"),
            Self::Internal(msg) => write!(f, "internal: {msg}"),
            Self::Database(err) => write!(f, "database error: {err}"),
        }
    }
}

impl std::error::Error for DashboardAuthServiceError {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeResponse {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<entity::employee::EmployeeRow> for EmployeeResponse {
    fn from(m: entity::employee::EmployeeRow) -> Self {
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
pub struct DashboardAuthResponse {
    pub employee: EmployeeResponse,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DashboardRefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DashboardRegisterResponse {
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DashboardVerifyEmailResponse {
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DashboardResendVerificationResponse {
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DashboardMeResponse {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl From<entity::employee::EmployeeRow> for DashboardMeResponse {
    fn from(m: entity::employee::EmployeeRow) -> Self {
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
pub trait DashboardAuthService: Send + Sync + 'static {
    async fn register(
        &self,
        email: &str,
        password: &str,
    ) -> Result<DashboardRegisterResponse, DashboardAuthServiceError>;
    async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> Result<DashboardAuthResponse, DashboardAuthServiceError>;
    async fn verify_email(
        &self,
        token: &str,
    ) -> Result<DashboardVerifyEmailResponse, DashboardAuthServiceError>;
    async fn resend_verification(
        &self,
        email: &str,
    ) -> Result<DashboardResendVerificationResponse, DashboardAuthServiceError>;
    async fn refresh(
        &self,
        token: &str,
    ) -> Result<DashboardRefreshResponse, DashboardAuthServiceError>;
    async fn logout(&self, token: &str) -> Result<(), DashboardAuthServiceError>;
    async fn get_me(
        &self,
        employee_id: Uuid,
    ) -> Result<DashboardMeResponse, DashboardAuthServiceError>;
}
