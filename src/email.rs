#[cfg(test)]
pub mod noop;
pub mod sender;
pub mod smtp;

#[cfg(test)]
pub use noop::NoopEmailSender;
pub use sender::EmailSender;
pub use smtp::SmtpEmailSender;

pub fn build_verification_email(
    app_name: &str,
    base_url: &str,
    token_hex: &str,
) -> (String, String, String) {
    let base_url = base_url.trim_end_matches('/');
    let verification_url =
        format!("{base_url}/api/v1/dashboard/auth/email/verification?token={token_hex}");
    let subject = format!("Verify your email — {app_name}");

    let text_template = include_str!("email/templates/verify_email.txt");
    let html_template = include_str!("email/templates/verify_email.html");

    let text_body = text_template.replace("{{verification_url}}", &verification_url);
    let html_body = html_template.replace("{{verification_url}}", &verification_url);

    (subject, text_body, html_body)
}
