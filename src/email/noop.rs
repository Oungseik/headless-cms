use async_trait::async_trait;

use super::sender::{EmailError, EmailSender};

#[derive(Debug)]
pub struct NoopEmailSender;

#[async_trait]
impl EmailSender for NoopEmailSender {
    async fn send(
        &self,
        to: &str,
        subject: &str,
        _text_body: &str,
        _html_body: &str,
    ) -> Result<(), EmailError> {
        tracing::info!(to, subject, "email would be sent (noop sender)");
        Ok(())
    }
}
