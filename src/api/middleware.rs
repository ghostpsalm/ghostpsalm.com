use axum::{
    extract::{Request, State},
    http::{header, Method, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use jsonwebtoken::{Algorithm, Validation};

use crate::{api::auth::Claims, state::AppState};

pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Extract Bearer token from Authorization header.
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "missing or malformed Authorization header"})),
            )
        })?
        .to_owned();

    // Build validation — pin to EdDSA only; reject any other algorithm claim.
    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.set_issuer(&[&state.cfg.jwt_issuer]);
    validation.set_audience(&[&state.cfg.jwt_audience]);
    validation.validate_exp = true;
    validation.validate_nbf = true;
    // 30-second clock skew tolerance — tight enough to be meaningful.
    validation.leeway = 30;

    let claims = jsonwebtoken::decode::<Claims>(&token, &state.decoding_key, &validation)
        .map_err(|e| {
            tracing::warn!("JWT validation failed: {e}");
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "invalid or expired token"})),
            )
        })?
        .claims;

    // Check revocation list.
    let revoked = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM revoked_tokens WHERE jti = $1)",
    )
    .bind(&claims.jti)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("revocation DB check failed: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "internal error"})),
        )
    })?;

    if revoked {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "token has been revoked"})),
        ));
    }

    // Scope enforcement: mutating methods require write scope.
    let needs_write = !matches!(
        *req.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    if needs_write && claims.scope != "write" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "write scope required for this operation"})),
        ));
    }

    // Attach validated claims to request extensions for handlers to use if needed.
    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}
