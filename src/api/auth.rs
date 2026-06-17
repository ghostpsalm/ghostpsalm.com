use axum::{extract::State, http::StatusCode, Extension, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub passphrase: String,
    /// "read" or "write". Defaults to "write".
    pub scope: Option<String>,
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
    pub sub: String,
    pub iss: String,
    pub aud: Vec<String>,
    pub jti: String,
    pub iat: i64,
    pub nbf: i64,
    pub exp: i64,
    /// "read" or "write"
    pub scope: String,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub sub: String,
    pub scope: String,
    pub jti: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub revoked: bool,
}

pub async fn issue_token(
    State(state): State<AppState>,
    Json(req): Json<AuthRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Rate limit — returns Err if quota exceeded.
    if state.auth_limiter.check().is_err() {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({"error": "too many authentication attempts — try again shortly"})),
        ));
    }

    // Constant-time passphrase compare.
    if !constant_time_eq(
        state.cfg.ghostpsalm_passphrase.as_bytes(),
        req.passphrase.as_bytes(),
    ) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "invalid passphrase"})),
        ));
    }

    // Validate and normalise scope.
    let scope = match req.scope.as_deref().unwrap_or("write") {
        "read" => "read",
        "write" => "write",
        other => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("invalid scope '{}' — must be 'read' or 'write'", other)
                })),
            ));
        }
    };

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
        scope: scope.to_string(),
    };

    let token =
        jsonwebtoken::encode(&Header::new(Algorithm::EdDSA), &claims, &state.encoding_key)
            .map_err(|e| {
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

/// Inspect the current token's claims. Useful for phone/assistant to verify
/// what scope they hold and how long the token has left.
pub async fn me(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<MeResponse>, StatusCode> {
    let revoked = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM revoked_tokens WHERE jti = $1)",
    )
    .bind(&claims.jti)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let expires_at = chrono::DateTime::from_timestamp(claims.exp, 0)
        .unwrap_or_else(Utc::now);

    Ok(Json(MeResponse {
        sub: claims.sub,
        scope: claims.scope,
        jti: claims.jti,
        expires_at,
        revoked,
    }))
}

/// Revoke a token by its jti.
pub async fn revoke_token(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("INSERT INTO revoked_tokens (jti) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(&claims.jti)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        let _ = a.iter().zip(a.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y));
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
