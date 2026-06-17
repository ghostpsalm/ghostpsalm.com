use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub vault_path: String,
    pub token_ttl_secs: u64,
    pub ghostpsalm_passphrase: String,
    pub private_key_path: String,
    pub public_key_path: String,
    pub jwt_issuer: String,
    pub jwt_audience: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();
        let cfg = config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .set_default("bind_addr", "127.0.0.1:3000")?
            .set_default("vault_path", "./vault")?
            .set_default("token_ttl_secs", 3600u64)?
            .set_default("jwt_issuer", "ghostpsalm")?
            .set_default("jwt_audience", "ghostpsalm-api")?
            .build()?
            .try_deserialize()?;
        Ok(cfg)
    }
}
