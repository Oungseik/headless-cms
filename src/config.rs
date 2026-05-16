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
    #[clap(long, env, default_value_t = 259200)]
    pub invitation_token_ttl: u64,
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
    #[clap(long, env)]
    pub smtp_host: String,
    #[clap(long, env, default_value_t = 587)]
    pub smtp_port: u16,
    #[clap(long, env)]
    pub smtp_username: String,
    #[clap(long, env)]
    pub smtp_password: String,
    #[clap(long, env)]
    pub smtp_from: String,
    #[clap(long, env, default_value = "Headless CMS")]
    pub smtp_from_name: String,
    #[clap(long, env, default_value_t = true)]
    pub smtp_starttls: bool,
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
