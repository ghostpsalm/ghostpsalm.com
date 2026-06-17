use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::config::Config;
use crate::vault;

#[derive(Debug, Deserialize)]
pub struct SaveKnowledgeRequest {
    pub title: String,
    pub content: String,
    /// Relative path within vault, e.g. "projects/dragoon" or "decisions"
    pub vault_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SaveKnowledgeResponse {
    pub path: String,
}

pub async fn save(
    State((_db, cfg)): State<(PgPool, Config)>,
    Json(req): Json<SaveKnowledgeRequest>,
) -> Result<Json<SaveKnowledgeResponse>, StatusCode> {
    let path = vault::knowledge::save_note(
        &cfg.vault_path,
        req.vault_path.as_deref().unwrap_or("notes"),
        &req.title,
        &req.content,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SaveKnowledgeResponse { path }))
}
