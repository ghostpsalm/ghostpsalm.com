use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;

use crate::git::{self, SyncResult};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub message: Option<String>,
}

pub async fn sync(
    State(state): State<AppState>,
    Json(req): Json<SyncRequest>,
) -> Result<Json<SyncResult>, (StatusCode, Json<serde_json::Value>)> {
    let msg = req.message.unwrap_or_else(|| {
        format!("assistant: sync {}", chrono::Local::now().format("%Y-%m-%d %H:%M"))
    });

    let result = git::sync_vault(&state.cfg.vault_path, &msg).await.map_err(|e| {
        tracing::error!(error = %e, "vault sync failed");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    if result.conflicts {
        tracing::warn!(
            conflict_files = ?result.conflict_files,
            "sync completed with conflicts — manual resolution required"
        );
    }

    Ok(Json(result))
}
