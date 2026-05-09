#[cfg(test)]
pub mod tests {
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    use crate::features::users::service::{UserService, UserServiceError};

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
        async fn get_by_id(&self, id: i32) -> Result<Option<entity::user::Model>, UserServiceError> {
            let users = self.users.lock().expect("mock users mutex poisoned");
            Ok(users.get(&id).cloned())
        }
    }
}
