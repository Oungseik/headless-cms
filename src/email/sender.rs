pub trait EmailSender: Send + Sync + 'static {
    async fn send(
        &self,
        to: &str,
        subject: &str,
        text_body: &str,
        html_body: &str,
    ) -> Result<(), EmailError>;
}

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("failed to send email: {0}")]
    Send(String),
}
