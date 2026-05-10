use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use chrono::NaiveDateTime;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app::AppState;
use crate::app::error::{AppError, AppResult, ErrorResponse};

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub created_at: NaiveDateTime,
}

impl From<entity::user::UserRow> for UserResponse {
    fn from(model: entity::user::UserRow) -> Self {
        Self {
            id: model.id,
            email: model.email,
            created_at: model.created_at,
        }
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    operation_id = "get_user_by_id",
    description = "Get a user by ID",
    responses(
        (status = 200, description = "Get the user information", body = UserResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Users",
)]
#[tracing::instrument]
pub async fn handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    let user = state
        .user_service
        .get_by_id(id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(UserResponse::from(user)))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::extract::{Path, State};
    use uuid::Uuid;

    use crate::app::AppState;
    use crate::features::auth::service::AuthService;
    use crate::features::auth::service_mock::tests::MockAuthService;
    use crate::features::users::service::UserService;
    use crate::features::users::service_mock::tests::MockUserService;

    #[tokio::test]
    async fn test_get_user_by_id_found() {
        let mock = MockUserService::new();
        let user_id = Uuid::now_v7();
        let now = chrono::Utc::now().naive_utc();
        let test_user = entity::user::UserRow {
            id: user_id,
            email: "test@example.com".into(),
            password_hash: String::new(),
            role: "customer".into(),
            is_active: true,
            email_verified_at: None,
            updated_at: now,
            created_at: now,
        };
        mock.users
            .lock()
            .expect("mock users mutex poisoned")
            .insert(user_id, test_user.clone());

        let mock: Arc<dyn UserService> = Arc::new(mock);
        let auth_service: Arc<dyn AuthService> = Arc::new(MockAuthService::new());
        let state = Arc::new(AppState {
            db: sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
            user_service: mock,
            auth_service,
        });

        let result = super::handler(State(state), Path(user_id)).await;
        let response = result.unwrap_or_else(|e| panic!("handler should return Ok: {e:?}"));
        let user = response.0;

        assert_eq!(user.id, user_id);
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_get_user_by_id_not_found() {
        let mock: Arc<dyn UserService> = Arc::new(MockUserService::new());
        let auth_service: Arc<dyn AuthService> = Arc::new(MockAuthService::new());
        let state = Arc::new(AppState {
            db: sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
            user_service: mock,
            auth_service,
        });

        let result = super::handler(State(state), Path(Uuid::now_v7())).await;
        assert!(result.is_err());
    }
}
