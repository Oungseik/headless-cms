use async_trait::async_trait;
use sea_orm::DbErr;

#[async_trait]
pub trait UserService: Send + Sync + 'static {
    async fn get_by_id(&self, id: i32) -> Result<Option<entity::user::Model>, DbErr>;
}
