use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Serialize;

use crate::app::AppState;
use crate::app::error::{AppError, AppResult};

#[derive(Serialize)]
pub struct VerifyAllResponse {
    pub message: String,
}

#[tracing::instrument]
pub async fn handler(State(state): State<Arc<AppState>>) -> AppResult<Json<VerifyAllResponse>> {
    let now = chrono::Utc::now().fixed_offset();

    entity::user::Entity::update_many()
        .filter(entity::user::Column::EmailVerifiedAt.is_null())
        .set(entity::user::ActiveModel {
            email_verified_at: Set(Some(now)),
            ..Default::default()
        })
        .exec(&state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    Ok(Json(VerifyAllResponse {
        message: "All users verified".to_string(),
    }))
}
