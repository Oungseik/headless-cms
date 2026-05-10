use async_trait::async_trait;

#[async_trait]
pub trait EmailService: Send + Sync + std::fmt::Debug {
    async fn send_verification_email(
        &self,
        to: &str,
        verification_link: &str,
    ) -> Result<(), String>;
}

#[derive(Clone, Debug)]
pub struct ConsoleEmailService;

#[async_trait]
impl EmailService for ConsoleEmailService {
    async fn send_verification_email(
        &self,
        to: &str,
        verification_link: &str,
    ) -> Result<(), String> {
        println!(
            "[EMAIL] To: {}\n[EMAIL] Verification link: {}\n",
            to, verification_link
        );
        Ok(())
    }
}
