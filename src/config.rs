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

    // Notifications
    /// Set false to disable all outbound notifications without removing keys.
    pub pushover_enabled: bool,
    pub pushover_token: Option<String>,
    pub pushover_user: Option<String>,
    /// IANA timezone name, e.g. "Australia/Perth". All cron times are in this zone.
    pub scheduler_tz: String,
    /// Cron expression (local time) for lunchtime review. Default: 12:30 daily.
    pub lunch_review_cron: String,
    /// Cron expression (local time) for evening errand reminder. Default: 17:30 daily.
    pub evening_errands_cron: String,
    /// Cron expression (local time) for morning brief. Default: 08:00 daily.
    pub morning_brief_cron: String,
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
            .set_default("pushover_enabled", false)?
            .set_default("scheduler_tz", "UTC")?
            .set_default("lunch_review_cron", "0 30 12 * * *")?
            .set_default("evening_errands_cron", "0 30 17 * * *")?
            .set_default("morning_brief_cron", "0 0 8 * * *")?
            .build()?
            .try_deserialize()?;
        Ok(cfg)
    }

    pub fn pushover_ready(&self) -> bool {
        self.pushover_enabled
            && self.pushover_token.is_some()
            && self.pushover_user.is_some()
    }
}
