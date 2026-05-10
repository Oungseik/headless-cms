use async_trait::async_trait;
use sea_query::{Expr, ExprTrait, Query, SqliteQueryBuilder};
use sea_query_sqlx::SqlxBinder;
use sqlx::SqlitePool;

use super::service::{UserService, UserServiceError};

#[derive(Clone, Debug)]
pub struct UserServiceImpl {
    pub db: SqlitePool,
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn get_by_id(&self, id: i32) -> Result<Option<entity::user::UserRow>, UserServiceError> {
        use entity::user::User;

        let (sql, values) = Query::select()
            .columns([
                User::Id,
                User::Email,
                User::PasswordHash,
                User::Role,
                User::IsActive,
                User::EmailVerifiedAt,
                User::CreatedAt,
                User::UpdatedAt,
            ])
            .from(User::Table)
            .and_where(Expr::col(User::Id).eq(id))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_as_with::<_, entity::user::UserRow, _>(&sql, values)
            .fetch_optional(&self.db)
            .await
            .map_err(UserServiceError::Database)
    }
}
