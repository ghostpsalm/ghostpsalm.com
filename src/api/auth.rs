use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub passphrase: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub jti: String,
    pub exp: i64,
    pub scope: String, // "read" | "write"
}

pub async fn issue_token(
    State((_, cfg)): State<(PgPool, Config)>,
    Json(req): Json<AuthRequest>,
) -> Result<Json<TokenResponse>, StatusCode> {
    // Constant-time compare via argon2 verify in production; passphrase check placeholder.
    if req.passphrase != std::env::var("GHOSTPSALM_PASSPHRASE").unwrap_or_default() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let expires_at = Utc::now() + Duration::seconds(cfg.token_ttl_secs as i64);
    let claims = Claims {
        sub: "assistant".to_string(),
        jti: Uuid::new_v4().to_string(),
        exp: expires_at.timestamp(),
        scope: "write".to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TokenResponse { token, expires_at }))
}
