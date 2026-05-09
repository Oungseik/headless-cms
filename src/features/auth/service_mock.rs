#[cfg(test)]
pub mod tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::Mutex;

    use async_trait::async_trait;

    use crate::features::auth::service::{
        AuthResponse, AuthService, AuthServiceError, MeResponse, RefreshResponse, RegisterResponse,
        ResendVerificationResponse, UserResponse, VerifyEmailResponse,
    };

    #[derive(Debug)]
    pub struct MockAuthService {
        pub users: Mutex<HashMap<String, entity::user::Model>>,
        pub refresh_tokens: Mutex<HashMap<String, i32>>,
        pub verified_emails: Mutex<HashSet<String>>,
        pub verification_tokens: Mutex<HashMap<String, String>>,
        pub next_id: Mutex<i32>,
    }

    impl MockAuthService {
        pub fn new() -> Self {
            Self {
                users: Mutex::new(HashMap::new()),
                refresh_tokens: Mutex::new(HashMap::new()),
                verified_emails: Mutex::new(HashSet::new()),
                verification_tokens: Mutex::new(HashMap::new()),
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
            email: &str,
            password: &str,
            role: &str,
        ) -> Result<RegisterResponse, AuthServiceError> {
            let mut users = self.users.lock().expect("mutex poisoned");
            if users.contains_key(email) {
                return Err(AuthServiceError::Conflict(
                    "Email already registered".to_string(),
                ));
            }

            let mut next_id = self.next_id.lock().expect("mutex poisoned");
            let id = *next_id;
            *next_id += 1;

            let now = chrono::Utc::now().naive_utc();
            let user = entity::user::Model {
                id,
                email: email.to_string(),
                password_hash: format!("hashed_{password}"),
                role: role.to_string(),
                is_active: true,
                email_verified_at: None,
                updated_at: now,
                created_at: now,
            };

            users.insert(email.to_string(), user);

            Ok(RegisterResponse {
                message: "Please check your email to verify your account.".to_string(),
            })
        }

        async fn login(
            &self,
            email: &str,
            password: &str,
        ) -> Result<AuthResponse, AuthServiceError> {
            let users = self.users.lock().expect("mutex poisoned");
            let user = users.get(email).ok_or_else(|| {
                AuthServiceError::NotFound("Invalid email or password".to_string())
            })?;

            if !user.is_active {
                return Err(AuthServiceError::Unauthorized(
                    "Account is deactivated".to_string(),
                ));
            }

            let expected_hash = format!("hashed_{password}");
            if user.password_hash != expected_hash {
                return Err(AuthServiceError::Unauthorized(
                    "Invalid email or password".to_string(),
                ));
            }

            let verified_emails = self.verified_emails.lock().expect("mutex poisoned");
            if !verified_emails.contains(email) {
                return Err(AuthServiceError::NotVerified(
                    "Email not verified. Please check your email.".to_string(),
                ));
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

        async fn verify_email(&self, token: &str) -> Result<VerifyEmailResponse, AuthServiceError> {
            let mut verification_tokens = self.verification_tokens.lock().expect("mutex poisoned");
            let email = verification_tokens.remove(token).ok_or_else(|| {
                AuthServiceError::NotFound("Invalid verification token".to_string())
            })?;

            let mut verified_emails = self.verified_emails.lock().expect("mutex poisoned");
            verified_emails.insert(email);

            Ok(VerifyEmailResponse {
                message: "Email verified successfully. You can now log in.".to_string(),
            })
        }

        async fn resend_verification(
            &self,
            _email: &str,
        ) -> Result<ResendVerificationResponse, AuthServiceError> {
            Ok(ResendVerificationResponse {
                message: "If an account with that email exists and is not yet verified, we've sent a verification email.".to_string(),
            })
        }

        async fn refresh(&self, token: &str) -> Result<RefreshResponse, AuthServiceError> {
            let tokens = self.refresh_tokens.lock().expect("mutex poisoned");
            let token_hash = Self::hash_token(token);
            let user_id = tokens.get(&token_hash).ok_or_else(|| {
                AuthServiceError::Unauthorized("Invalid refresh token".to_string())
            })?;
            let user_id = *user_id;
            drop(tokens);

            let users = self.users.lock().expect("mutex poisoned");
            let user = users
                .values()
                .find(|u| u.id == user_id)
                .ok_or_else(|| AuthServiceError::NotFound("User not found".to_string()))?;
            if !user.is_active {
                return Err(AuthServiceError::Unauthorized(
                    "Account is deactivated".to_string(),
                ));
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

        async fn logout(&self, token: &str) -> Result<(), AuthServiceError> {
            let token_hash = Self::hash_token(token);
            let mut tokens = self.refresh_tokens.lock().expect("mutex poisoned");
            tokens.remove(&token_hash);
            Ok(())
        }

        async fn get_me(&self, user_id: i32) -> Result<MeResponse, AuthServiceError> {
            let users = self.users.lock().expect("mutex poisoned");
            let user = users
                .values()
                .find(|u| u.id == user_id)
                .ok_or_else(|| AuthServiceError::Unauthorized("User not found".to_string()))?;
            Ok(MeResponse::from(user.clone()))
        }
    }
}
