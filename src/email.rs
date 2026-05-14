pub mod noop;
pub mod sender;
pub mod smtp;

use std::sync::Arc;

pub use noop::NoopEmailSender;
pub use sender::EmailSender;
pub use smtp::SmtpEmailSender;

use crate::config::{AppEnv, Config};

pub fn build_email_sender(config: &Config) -> Arc<dyn EmailSender> {
    if config.app_env == AppEnv::Development && config.smtp_host.is_empty() {
        tracing::info!("SMTP not configured in development, using noop email sender");
        return Arc::new(NoopEmailSender);
    }

    match SmtpEmailSender::new(
        &config.smtp_host,
        config.smtp_port,
        &config.smtp_username,
        &config.smtp_password,
        &config.smtp_from_name,
        &config.smtp_from,
        config.smtp_starttls,
    ) {
        Ok(sender) => Arc::new(sender),
        Err(e) => {
            assert!(config.app_env == AppEnv::Production);
            tracing::warn!("failed to create SMTP sender, falling back to noop: {e}");
            Arc::new(NoopEmailSender)
        }
    }
}

pub fn build_verification_email(
    app_name: &str,
    base_url: &str,
    token_hex: &str,
) -> (String, String, String) {
    let base_url = base_url.trim_end_matches('/');
    let verification_url =
        format!("{base_url}/api/v1/dashboard/auth/verify-email?token={token_hex}");
    let subject = format!("Verify your email — {app_name}");

    let text_template = include_str!("email/templates/verify_email.txt");
    let html_template = include_str!("email/templates/verify_email.html");

    let text_body = text_template.replace("{{verification_url}}", &verification_url);
    let html_body = html_template.replace("{{verification_url}}", &verification_url);

    (subject, text_body, html_body)
}
