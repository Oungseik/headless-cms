use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::features::auth::service::AuthServiceError;
use crate::features::users::service::UserServiceError;

/// use to generate open api schema
#[derive(ToSchema, Serialize)]
pub struct ErrorResponse {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug)]
pub enum AppError {
    InternalServerError,
    NotFound,
    Unauthorized,
    Forbidden(String),
    BadRequest(String),
    Conflict(String),
}

impl From<UserServiceError> for AppError {
    fn from(err: UserServiceError) -> Self {
        match err {
            UserServiceError::NotFound(_) => Self::NotFound,
            UserServiceError::Database(db_err) => {
                tracing::error!(%db_err, "database error");
                Self::InternalServerError
            }
        }
    }
}

impl From<AuthServiceError> for AppError {
    fn from(err: AuthServiceError) -> Self {
        match err {
            AuthServiceError::NotFound(_) => Self::NotFound,
            AuthServiceError::Unauthorized(_) => Self::Unauthorized,
            AuthServiceError::NotVerified(msg) => Self::Forbidden(msg),
            AuthServiceError::Conflict(msg) => Self::Conflict(msg),
            AuthServiceError::Internal(msg) => {
                tracing::error!(%msg, "internal error");
                Self::InternalServerError
            }
            AuthServiceError::Database(db_err) => {
                tracing::error!(%db_err, "database error");
                Self::InternalServerError
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::InternalServerError => {
                tracing::error!("responding with internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        message: "Internal server error".into(),
                        details: None,
                    }),
                )
            }
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    message: "Not found".into(),
                    details: None,
                }),
            ),
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    message: "Unauthorized".into(),
                    details: None,
                }),
            ),
            Self::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    message: msg,
                    details: None,
                }),
            ),
            Self::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    message: msg,
                    details: None,
                }),
            ),
            Self::Conflict(msg) => (
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    message: msg,
                    details: None,
                }),
            ),
        }
        .into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
