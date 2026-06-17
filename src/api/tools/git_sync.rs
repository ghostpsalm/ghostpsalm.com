use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::config::Config;
use crate::git;

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub committed: bool,
    pub pushed: bool,
    pub message: String,
}

pub async fn sync(
    State((_db, cfg)): State<(PgPool, Config)>,
    Json(req): Json<SyncRequest>,
) -> Result<Json<SyncResponse>, StatusCode> {
    let msg = req
        .message
        .unwrap_or_else(|| format!("assistant: sync {}", chrono::Local::now().format("%Y-%m-%d %H:%M")));

    let result = git::sync_vault(&cfg.vault_path, &msg)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
}
