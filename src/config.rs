use std::sync::OnceLock;

use clap::Parser;

#[derive(Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum AppEnv {
    Development,
    Testing,
    Production,
}

/// Application configuration loaded from environment variables or CLI flags.
#[derive(clap::Parser, Clone)]
pub struct Config {
    #[clap(long, env = "APP_ENV", default_value = "development", value_enum)]
    pub app_env: AppEnv,
    #[clap(long, env, default_value = "sqlite::memory:")]
    pub database_url: String,
    #[clap(long, env, default_value = "0.0.0.0:5000")]
    pub address: String,
    #[clap(long, env, default_value = "change-me-in-production")]
    pub jwt_secret: String,
    #[clap(long, env, default_value_t = 900)]
    pub access_token_ttl: u64,
    #[clap(long, env, default_value_t = 604800)]
    pub refresh_token_ttl: u64,
    #[clap(long, env, default_value = "http://localhost:3000")]
    pub base_url: String,
    #[clap(long, env, default_value = "http://localhost:3000")]
    pub allowed_origins: String,
    #[clap(long, env, default_value_t = 86400)]
    pub email_verification_token_ttl: u64,
    #[clap(long, env, default_value_t = 12)]
    pub bcrypt_cost: u32,
    #[clap(long, env, default_value_t = true)]
    pub rate_limit_enabled: bool,
    #[clap(long, env, default_value_t = 10)]
    pub rate_limit_per_second: u64,
    #[clap(long, env, default_value_t = 10)]
    pub rate_limit_burst: u32,
    #[clap(long, env, default_value_t = 60)]
    pub login_rate_limit_per_second: u64,
    #[clap(long, env, default_value_t = 3)]
    pub login_rate_limit_burst: u32,
    #[clap(long, env, default_value = "Headless CMS")]
    pub app_name: String,
    #[clap(long, env, default_value = "")]
    pub smtp_host: String,
    #[clap(long, env, default_value_t = 587)]
    pub smtp_port: u16,
    #[clap(long, env, default_value = "")]
    pub smtp_username: String,
    #[clap(long, env, default_value = "")]
    pub smtp_password: String,
    #[clap(long, env, default_value = "noreply@localhost")]
    pub smtp_from: String,
    #[clap(long, env, default_value = "Headless CMS")]
    pub smtp_from_name: String,
    #[clap(long, env, default_value_t = true)]
    pub smtp_starttls: bool,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("app_env", &self.app_env)
            .field("database_url", &"[REDACTED]")
            .field("address", &self.address)
            .field("jwt_secret", &"[REDACTED]")
            .field("access_token_ttl", &self.access_token_ttl)
            .field("refresh_token_ttl", &self.refresh_token_ttl)
            .field("base_url", &self.base_url)
            .field("allowed_origins", &self.allowed_origins)
            .field(
                "email_verification_token_ttl",
                &self.email_verification_token_ttl,
            )
            .field("bcrypt_cost", &self.bcrypt_cost)
            .field("rate_limit_enabled", &self.rate_limit_enabled)
            .field("rate_limit_per_second", &self.rate_limit_per_second)
            .field("rate_limit_burst", &self.rate_limit_burst)
            .field(
                "login_rate_limit_per_second",
                &self.login_rate_limit_per_second,
            )
            .field("login_rate_limit_burst", &self.login_rate_limit_burst)
            .field("app_name", &self.app_name)
            .field("smtp_host", &self.smtp_host)
            .field("smtp_port", &self.smtp_port)
            .field("smtp_username", &self.smtp_username)
            .field("smtp_password", &"[REDACTED]")
            .field("smtp_from", &self.smtp_from)
            .field("smtp_from_name", &self.smtp_from_name)
            .field("smtp_starttls", &self.smtp_starttls)
            .finish()
    }
}

/// Returns a static reference to the global [`Config`].
///
/// Loads `.env` on first call via `dotenvy`, then parses CLI/env flags.
/// Subsequent calls return the same instance.
pub fn get_config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        dotenvy::dotenv().ok();
        Config::parse()
    })
}
