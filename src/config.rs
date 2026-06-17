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

    // TLS — if both are set the service runs HTTPS; otherwise HTTP.
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,

    // Auth rate limiting — global requests-per-minute cap on /auth/token.
    pub auth_rate_limit_rpm: u32,

    // Notifications
    pub pushover_enabled: bool,
    pub pushover_token: Option<String>,
    pub pushover_user: Option<String>,
    pub scheduler_tz: String,
    pub lunch_review_cron: String,
    pub evening_errands_cron: String,
    pub morning_brief_cron: String,

    // Vault auto-sync
    pub vault_sync_enabled: bool,
    pub vault_sync_cron: String,
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
            .set_default("auth_rate_limit_rpm", 10u32)?
            .set_default("pushover_enabled", false)?
            .set_default("scheduler_tz", "UTC")?
            .set_default("lunch_review_cron", "0 30 12 * * *")?
            .set_default("evening_errands_cron", "0 30 17 * * *")?
            .set_default("morning_brief_cron", "0 0 8 * * *")?
            .set_default("vault_sync_enabled", true)?
            .set_default("vault_sync_cron", "0 0 */4 * * *")?
            .build()?
            .try_deserialize()?;
        Ok(cfg)
    }

    pub fn pushover_ready(&self) -> bool {
        self.pushover_enabled
            && self.pushover_token.is_some()
            && self.pushover_user.is_some()
    }

    pub fn tls_enabled(&self) -> bool {
        self.tls_cert_path.is_some() && self.tls_key_path.is_some()
    }
}
