use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use serde::Serialize;

use crate::app::AppState;
use crate::app::error::{AppError, AppResult};

#[derive(Serialize)]
pub struct VerifyAllResponse {
    pub message: String,
}

#[tracing::instrument]
pub async fn handler(State(state): State<Arc<AppState>>) -> AppResult<Json<VerifyAllResponse>> {
    use entity::employee::Employee;

    let now = chrono::Utc::now().fixed_offset();

    let (sql, values) = Query::update()
        .table(Employee::Table)
        .values([(Employee::EmailVerifiedAt, now.to_rfc3339().into())])
        .and_where(Expr::col(Employee::EmailVerifiedAt).is_null())
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_with(&sql, values)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    Ok(Json(VerifyAllResponse {
        message: "All employees verified".to_string(),
    }))
}
