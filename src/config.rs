use clap::Parser;
use std::sync::OnceLock;

#[derive(clap::Parser)]
pub struct Config {
    #[clap(long, env)]
    pub database_url: String,
    #[clap(long, env, default_value = "0.0.0.0:5000")]
    pub address: String,
    #[clap(long, env, default_value = "change-me-in-production")]
    pub jwt_secret: String,
    #[clap(long, env, default_value_t = 900)]
    pub access_token_ttl: u64,
    #[clap(long, env, default_value_t = 604800)]
    pub refresh_token_ttl: u64,
}

pub fn get_config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        dotenv::dotenv().ok();
        Config::parse()
    })
}
