use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::git;
use crate::state::AppState;

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
    State(state): State<AppState>,
    Json(req): Json<SyncRequest>,
) -> Result<Json<SyncResponse>, StatusCode> {
    let msg = req.message.unwrap_or_else(|| {
        format!(
            "assistant: sync {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M")
        )
    });

    let result = git::sync_vault(&state.cfg.vault_path, &msg)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
}
