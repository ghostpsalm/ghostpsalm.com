use std::sync::Arc;

use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cfg: Config,
    pub encoding_key: Arc<EncodingKey>,
    pub decoding_key: Arc<DecodingKey>,
}

impl AppState {
    pub fn new(
        db: PgPool,
        cfg: Config,
        encoding_key: EncodingKey,
        decoding_key: DecodingKey,
    ) -> Self {
        Self {
            db,
            cfg,
            encoding_key: Arc::new(encoding_key),
            decoding_key: Arc::new(decoding_key),
        }
    }
}
