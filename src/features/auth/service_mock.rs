#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use async_trait::async_trait;

    use crate::features::auth::service::{
        AuthService, AuthServiceError, AuthResponse, RefreshResponse, UserResponse,
    };

    #[derive(Debug)]
    pub struct MockAuthService {
        pub users: Mutex<HashMap<String, entity::user::Model>>,
        pub refresh_tokens: Mutex<HashMap<String, i32>>,
        pub next_id: Mutex<i32>,
    }

    impl MockAuthService {
        pub fn new() -> Self {
            Self {
                users: Mutex::new(HashMap::new()),
                refresh_tokens: Mutex::new(HashMap::new()),
                next_id: Mutex::new(1),
            }
        }

        fn hash_token(token: &str) -> String {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(token.as_bytes());
            format!("{:x}", hasher.finalize())
        }
    }

    #[async_trait]
    impl AuthService for MockAuthService {
        async fn register(
            &self,
            username: String,
            email: String,
            password: String,
            role: String,
        ) -> Result<AuthResponse, AuthServiceError> {
            let mut users = self.users.lock().expect("mutex poisoned");
            if users.contains_key(&username) {
                return Err(AuthServiceError::Conflict(format!(
                    "username '{username}' already exists"
                )));
            }

            let mut next_id = self.next_id.lock().expect("mutex poisoned");
            let id = *next_id;
            *next_id += 1;

            let now = chrono::Utc::now().naive_utc();
            let user = entity::user::Model {
                id,
                username: username.clone(),
                email,
                password_hash: format!("hashed_{password}"),
                role,
                is_active: true,
                created_at: now,
                updated_at: now,
            };

            users.insert(username.clone(), user.clone());

            let access_token = format!("access_{id}");
            let refresh_token = format!("refresh_{id}");

            let mut tokens = self.refresh_tokens.lock().expect("mutex poisoned");
            tokens.insert(Self::hash_token(&refresh_token), id);

            Ok(AuthResponse {
                user: UserResponse::from(user),
                access_token,
                refresh_token,
            })
        }

        async fn login(
            &self,
            username: String,
            password: String,
        ) -> Result<AuthResponse, AuthServiceError> {
            let users = self.users.lock().expect("mutex poisoned");
            let user = users.get(&username).ok_or(AuthServiceError::NotFound)?;

            if !user.is_active {
                return Err(AuthServiceError::Unauthorized);
            }

            let expected_hash = format!("hashed_{password}");
            if user.password_hash != expected_hash {
                return Err(AuthServiceError::Unauthorized);
            }

            let id = user.id;
            let user = user.clone();
            drop(users);

            let access_token = format!("access_{id}");
            let refresh_token = format!("refresh_{id}_{}", chrono::Utc::now().timestamp());

            let mut tokens = self.refresh_tokens.lock().expect("mutex poisoned");
            tokens.insert(Self::hash_token(&refresh_token), id);

            Ok(AuthResponse {
                user: UserResponse::from(user),
                access_token,
                refresh_token,
            })
        }

        async fn refresh(&self, token: String) -> Result<RefreshResponse, AuthServiceError> {
            let tokens = self.refresh_tokens.lock().expect("mutex poisoned");
            let token_hash = Self::hash_token(&token);
            let user_id = tokens
                .get(&token_hash)
                .ok_or(AuthServiceError::Unauthorized)?;
            let user_id = *user_id;
            drop(tokens);

            let users = self.users.lock().expect("mutex poisoned");
            let user = users
                .values()
                .find(|u| u.id == user_id)
                .ok_or(AuthServiceError::NotFound)?;
            if !user.is_active {
                return Err(AuthServiceError::Unauthorized);
            }

            let mut tokens = self.refresh_tokens.lock().expect("mutex poisoned");
            tokens.remove(&token_hash);

            let access_token = format!("access_{user_id}");
            let new_refresh_token = format!("refresh_{user_id}_{}", chrono::Utc::now().timestamp());
            tokens.insert(Self::hash_token(&new_refresh_token), user_id);

            Ok(RefreshResponse {
                access_token,
                refresh_token: new_refresh_token,
            })
        }

        async fn logout(&self, token: String) -> Result<(), AuthServiceError> {
            let token_hash = Self::hash_token(&token);
            let mut tokens = self.refresh_tokens.lock().expect("mutex poisoned");
            tokens.remove(&token_hash);
            Ok(())
        }
    }
}
