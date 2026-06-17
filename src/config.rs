use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub vault_path: String,
    pub jwt_secret: String,
    /// Token lifetime in seconds (default 3600)
    pub token_ttl_secs: u64,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();
        let cfg = config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .set_default("bind_addr", "127.0.0.1:3000")?
            .set_default("vault_path", "./vault")?
            .set_default("token_ttl_secs", 3600u64)?
            .build()?
            .try_deserialize()?;
        Ok(cfg)
    }
}
