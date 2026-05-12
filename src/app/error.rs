use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
#[expect(dead_code, reason = "variants will be used by future handlers")]
pub enum AppError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("internal server error")]
    InternalServerError,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, String::new()),
            Self::Forbidden => (StatusCode::FORBIDDEN, String::new()),
            Self::NotFound => (StatusCode::NOT_FOUND, String::new()),
            Self::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            Self::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, String::new()),
        };

        let body = ErrorResponse {
            error: status.canonical_reason().unwrap_or("Unknown").to_string(),
            message,
        };

        (status, axum::Json(body)).into_response()
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        tracing::error!("io error: {err}");
        Self::InternalServerError
    }
}

impl From<crate::features::dashboard_auth::service::DashboardAuthServiceError> for AppError {
    fn from(err: crate::features::dashboard_auth::service::DashboardAuthServiceError) -> Self {
        use crate::features::dashboard_auth::service::DashboardAuthServiceError as E;
        match err {
            E::OwnerAlreadyExists => {
                Self::Conflict("An owner has already been registered".to_string())
            }
            E::WeakPassword => {
                Self::BadRequest("Password must be at least 8 characters".to_string())
            }
            E::Database(e) => {
                tracing::error!("database error: {e}");
                Self::InternalServerError
            }
            E::PasswordHashing(e) => {
                tracing::error!("password hashing failed: {e}");
                Self::InternalServerError
            }
        }
    }
}
