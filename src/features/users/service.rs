use async_trait::async_trait;
use std::fmt;
use uuid::Uuid;

#[derive(Debug)]
pub enum UserServiceError {
    NotFound(Uuid),
    Database(sqlx::Error),
}

impl fmt::Display for UserServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "user with id {id} not found"),
            Self::Database(err) => write!(f, "database error: {err}"),
        }
    }
}

impl std::error::Error for UserServiceError {}

#[async_trait]
pub trait UserService: Send + Sync + 'static {
    async fn get_by_id(&self, id: Uuid) -> Result<Option<entity::user::UserRow>, UserServiceError>;
}
