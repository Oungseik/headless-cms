use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::features::users::service::UserServiceError;

/// use to generate open api schema
#[derive(ToSchema, Serialize)]
pub struct ErrorResponse {
    message: String,
}

#[derive(Debug)]
pub enum AppError {
    InternalServerError,
    NotFound,
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

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::InternalServerError => {
                tracing::error!("responding with internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        message: "Internal server error".into(),
                    }),
                )
            }
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    message: "Not found".into(),
                }),
            ),
        }
        .into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
