use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait};

use super::service::{UserService, UserServiceError};

#[derive(Clone, Debug)]
pub struct UserServiceImpl {
    pub db: DatabaseConnection,
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn get_by_id(&self, id: i32) -> Result<Option<entity::user::Model>, UserServiceError> {
        entity::user::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(UserServiceError::Database)
    }
}
