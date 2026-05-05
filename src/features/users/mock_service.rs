#[cfg(test)]
pub mod tests {
    use async_trait::async_trait;
    use sea_orm::DbErr;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use crate::features::users::service::UserService;

    #[derive(Debug)]
    pub struct MockUserService {
        pub users: Mutex<HashMap<i32, entity::user::Model>>,
    }

    impl MockUserService {
        pub fn new() -> Self {
            Self {
                users: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl UserService for MockUserService {
        async fn get_by_id(&self, id: i32) -> Result<Option<entity::user::Model>, DbErr> {
            let users = self.users.lock().unwrap();
            Ok(users.get(&id).cloned())
        }
    }
}
