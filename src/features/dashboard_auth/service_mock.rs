use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::email_service::EmailService;
use super::service::{
    DashboardAuthResponse, DashboardAuthService, DashboardAuthServiceError, DashboardMeResponse,
    DashboardRefreshResponse, DashboardRegisterResponse, DashboardResendVerificationResponse,
    DashboardVerifyEmailResponse, EmployeeResponse,
};

#[derive(Clone, Debug)]
pub struct MockDashboardAuthService {
    pub employees: Arc<Mutex<HashMap<String, MockEmployee>>>,
    pub email_service: Arc<dyn EmailService>,
}

#[derive(Debug, Clone)]
pub struct MockEmployee {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub email_verified_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl MockDashboardAuthService {
    pub fn new() -> Self {
        Self {
            employees: Arc::new(Mutex::new(HashMap::new())),
            email_service: Arc::new(MockEmailService),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MockEmailService;

#[async_trait]
impl EmailService for MockEmailService {
    async fn send_verification_email(&self, _to: &str, _link: &str) -> Result<(), String> {
        Ok(())
    }
}

#[async_trait]
impl DashboardAuthService for MockDashboardAuthService {
    async fn register(
        &self,
        email: &str,
        _password: &str,
    ) -> Result<DashboardRegisterResponse, DashboardAuthServiceError> {
        let mut employees = self.employees.lock().await;

        if !employees.is_empty() {
            return Err(DashboardAuthServiceError::Conflict(
                "An owner account already exists".to_string(),
            ));
        }

        if employees.contains_key(email) {
            return Err(DashboardAuthServiceError::Conflict(
                "Email already registered".to_string(),
            ));
        }

        let employee = MockEmployee {
            id: Uuid::now_v7(),
            email: email.to_string(),
            role: "owner".to_string(),
            is_active: true,
            email_verified_at: None,
            created_at: chrono::Utc::now(),
        };

        employees.insert(email.to_string(), employee);

        Ok(DashboardRegisterResponse {
            message: "Please check your email to verify your account.".to_string(),
        })
    }

    async fn login(
        &self,
        email: &str,
        _password: &str,
    ) -> Result<DashboardAuthResponse, DashboardAuthServiceError> {
        let employees = self.employees.lock().await;

        let employee = employees.get(email).ok_or_else(|| {
            DashboardAuthServiceError::Unauthorized("Invalid email or password".to_string())
        })?;

        if employee.email_verified_at.is_none() {
            return Err(DashboardAuthServiceError::NotVerified(
                "Email not verified. Please check your email.".to_string(),
            ));
        }

        if !employee.is_active {
            return Err(DashboardAuthServiceError::Unauthorized(
                "Account is deactivated".to_string(),
            ));
        }

        Ok(DashboardAuthResponse {
            employee: EmployeeResponse {
                id: employee.id,
                email: employee.email.clone(),
                role: employee.role.clone(),
                is_active: employee.is_active,
                created_at: employee.created_at,
            },
            access_token: "mock-access-token".to_string(),
            refresh_token: "mock-refresh-token".to_string(),
        })
    }

    async fn verify_email(
        &self,
        _token: &str,
    ) -> Result<DashboardVerifyEmailResponse, DashboardAuthServiceError> {
        Ok(DashboardVerifyEmailResponse {
            message: "Email verified successfully. You can now log in.".to_string(),
        })
    }

    async fn resend_verification(
        &self,
        _email: &str,
    ) -> Result<DashboardResendVerificationResponse, DashboardAuthServiceError> {
        Ok(DashboardResendVerificationResponse {
            message: format!(
                "If an account with that email exists and is not yet verified, we've sent a verification email."
            ),
        })
    }

    async fn refresh(
        &self,
        _token: &str,
    ) -> Result<DashboardRefreshResponse, DashboardAuthServiceError> {
        Ok(DashboardRefreshResponse {
            access_token: "mock-access-token".to_string(),
            refresh_token: "mock-refresh-token".to_string(),
        })
    }

    async fn logout(&self, _token: &str) -> Result<(), DashboardAuthServiceError> {
        Ok(())
    }

    async fn get_me(
        &self,
        employee_id: Uuid,
    ) -> Result<DashboardMeResponse, DashboardAuthServiceError> {
        let employees = self.employees.lock().await;

        let employee = employees
            .values()
            .find(|e| e.id == employee_id)
            .ok_or_else(|| DashboardAuthServiceError::NotFound("Employee not found".to_string()))?;

        Ok(DashboardMeResponse {
            id: employee.id,
            email: employee.email.clone(),
            role: employee.role.clone(),
            is_active: employee.is_active,
            email_verified_at: employee.email_verified_at,
            updated_at: employee.created_at,
            created_at: employee.created_at,
        })
    }
}
