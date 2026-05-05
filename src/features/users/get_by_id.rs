use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use serde::Serialize;
use utoipa::ToSchema;

use crate::app::AppState;
use crate::app::error::{AppError, AppResult, ErrorResponse};

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

impl From<entity::user::Model> for UserResponse {
    fn from(model: entity::user::Model) -> Self {
        Self {
            id: model.id,
            username: model.username,
            email: model.email,
            created_at: model.created_at.to_string(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
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
    Path(id): Path<i32>,
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

    use crate::app::AppState;
    use crate::features::users::mock_service::tests::MockUserService;
    use crate::features::users::service::UserService;

    #[tokio::test]
    async fn test_get_user_by_id_found() {
        let mock = MockUserService::new();
        let test_user = entity::user::Model {
            id: 1,
            username: "testuser".into(),
            email: "test@example.com".into(),
            created_at: chrono::Utc::now().naive_utc(),
        };
        mock.users.lock().unwrap().insert(1, test_user.clone());

        let mock: Arc<dyn UserService> = Arc::new(mock);

        let db = sea_orm::Database::connect("sqlite::memory:")
            .await
            .expect("failed to connect to test db");
        let state = Arc::new(AppState {
            user_service: mock,
            db,
        });

        let result = super::handler(State(state), Path(1)).await;
        let response = result.unwrap_or_else(|e| panic!("handler should return Ok: {e:?}"));
        let user = response.0;

        assert_eq!(user.id, 1);
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_get_user_by_id_not_found() {
        let mock: Arc<dyn UserService> = Arc::new(MockUserService::new());
        let db = sea_orm::Database::connect("sqlite::memory:")
            .await
            .expect("failed to connect to test db");
        let state = Arc::new(AppState {
            user_service: mock,
            db,
        });

        let result = super::handler(State(state), Path(999)).await;
        assert!(result.is_err());
    }
}
