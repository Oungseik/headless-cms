use std::sync::OnceLock;

use clap::Parser;

/// Application configuration loaded from environment variables or CLI flags.
#[derive(clap::Parser, Clone)]
pub struct Config {
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
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
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
