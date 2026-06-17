use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub passphrase: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub scope: String,
}

/// JWT claims. All standard fields present; `scope` is our custom extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — always "assistant" for now; extend when multi-user is needed.
    pub sub: String,
    /// Issuer — validated on decode.
    pub iss: String,
    /// Audience — validated on decode.
    pub aud: Vec<String>,
    /// JWT ID — used for revocation checks.
    pub jti: String,
    /// Issued-at (Unix seconds).
    pub iat: i64,
    /// Not-before (Unix seconds) — same as iat; prevents use of tokens issued in the future.
    pub nbf: i64,
    /// Expiry (Unix seconds).
    pub exp: i64,
    /// "read" or "write" — enforced by middleware.
    pub scope: String,
}

pub async fn issue_token(
    State(state): State<AppState>,
    Json(req): Json<AuthRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Constant-time compare to prevent timing attacks on the passphrase.
    let expected = state.cfg.ghostpsalm_passphrase.as_bytes();
    let provided = req.passphrase.as_bytes();

    let passphrase_ok = constant_time_eq(expected, provided);
    if !passphrase_ok {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "invalid passphrase"})),
        ));
    }

    let now = Utc::now();
    let expires_at = now + Duration::seconds(state.cfg.token_ttl_secs as i64);

    let claims = Claims {
        sub: "assistant".to_string(),
        iss: state.cfg.jwt_issuer.clone(),
        aud: vec![state.cfg.jwt_audience.clone()],
        jti: Uuid::new_v4().to_string(),
        iat: now.timestamp(),
        nbf: now.timestamp(),
        exp: expires_at.timestamp(),
        scope: "write".to_string(),
    };

    // EdDSA header — algorithm is explicit, not implicit.
    let header = Header::new(Algorithm::EdDSA);

    let token = jsonwebtoken::encode(&header, &claims, &state.encoding_key).map_err(|e| {
        tracing::error!("token signing failed: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "token signing failed"})),
        )
    })?;

    Ok(Json(TokenResponse {
        token,
        expires_at,
        scope: claims.scope,
    }))
}

/// Revoke a token by jti. Caller must supply the token they want revoked.
pub async fn revoke_token(
    State(state): State<AppState>,
    claims: axum::Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("INSERT INTO revoked_tokens (jti) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(&claims.jti)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Timing-safe byte comparison.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        // Still run the loop to avoid length-based timing leak.
        let _ = a.iter().zip(a.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y));
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
