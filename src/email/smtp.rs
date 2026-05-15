use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    transport::smtp::authentication::Credentials,
};

use super::sender::{EmailError, EmailSender};

pub struct SmtpEmailSender {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from_name: String,
    from_address: String,
}

impl std::fmt::Debug for SmtpEmailSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmtpEmailSender")
            .field("from_name", &self.from_name)
            .field("from_address", &self.from_address)
            .finish_non_exhaustive()
    }
}

impl SmtpEmailSender {
    pub fn new(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        from_name: &str,
        from_address: &str,
        tls: bool,
    ) -> Result<Self, EmailError> {
        let credentials = Credentials::new(username.to_string(), password.to_string());

        let mailer = if tls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
                .map_err(|e| EmailError::Send(format!("failed to create SMTP transport: {e}")))?
                .port(port)
                .credentials(credentials)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
                .port(port)
                .credentials(credentials)
                .build()
        };

        Ok(Self {
            mailer,
            from_name: from_name.to_string(),
            from_address: from_address.to_string(),
        })
    }
}

impl EmailSender for SmtpEmailSender {
    async fn send(
        &self,
        to: &str,
        subject: &str,
        text_body: &str,
        html_body: &str,
    ) -> Result<(), EmailError> {
        let from = format!("{} <{}>", self.from_name, self.from_address);

        let email = Message::builder()
            .from(
                from.parse()
                    .map_err(|e| EmailError::Send(format!("invalid from address: {e}")))?,
            )
            .to(to
                .parse()
                .map_err(|e| EmailError::Send(format!("invalid to address: {e}")))?)
            .subject(subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_PLAIN)
                            .body(text_body.to_string()),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(lettre::message::header::ContentType::TEXT_HTML)
                            .body(html_body.to_string()),
                    ),
            )
            .map_err(|e| EmailError::Send(format!("failed to build email: {e}")))?;

        self.mailer
            .send(email)
            .await
            .map_err(|e| EmailError::Send(format!("SMTP send failed: {e}")))?;

        Ok(())
    }
}
