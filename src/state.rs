use std::num::NonZeroU32;
use std::sync::Arc;

use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cfg: Config,
    pub encoding_key: Arc<EncodingKey>,
    pub decoding_key: Arc<DecodingKey>,
    pub http: Arc<reqwest::Client>,
    /// Global rate limiter for /auth/token — shared across all callers.
    pub auth_limiter: Arc<DefaultDirectRateLimiter>,
}

impl AppState {
    pub fn new(
        db: PgPool,
        cfg: Config,
        encoding_key: EncodingKey,
        decoding_key: DecodingKey,
    ) -> Self {
        let http = reqwest::Client::builder()
            .user_agent("ghostpsalm/0.1")
            .https_only(true)
            .build()
            .expect("failed to build HTTP client");

        let rpm = NonZeroU32::new(cfg.auth_rate_limit_rpm.max(1))
            .unwrap_or(NonZeroU32::new(10).unwrap());
        let auth_limiter = Arc::new(RateLimiter::direct(Quota::per_minute(rpm)));

        Self {
            db,
            cfg,
            encoding_key: Arc::new(encoding_key),
            decoding_key: Arc::new(decoding_key),
            http: Arc::new(http),
            auth_limiter,
        }
    }
}
