use async_trait::async_trait;

#[async_trait]
pub trait EmailService: Send + Sync + std::fmt::Debug {
    async fn send_verification_email(
        &self,
        to: &str,
        verification_link: &str,
    ) -> Result<(), String>;
}

#[derive(Debug)]
pub struct ConsoleEmailService;

#[async_trait]
impl EmailService for ConsoleEmailService {
    async fn send_verification_email(
        &self,
        to: &str,
        verification_link: &str,
    ) -> Result<(), String> {
        println!("[Email Service] Sending verification email to: {to}");
        println!("[Email Service] Verification link: {verification_link}");
        Ok(())
    }
}
