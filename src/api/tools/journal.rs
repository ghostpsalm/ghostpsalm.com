use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::vault;
use sqlx::PgPool;

#[derive(Debug, Deserialize)]
pub struct AppendJournalRequest {
    pub note: String,
    pub date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct AppendJournalResponse {
    pub path: String,
}

pub async fn append(
    State((_db, cfg)): State<(PgPool, Config)>,
    Json(req): Json<AppendJournalRequest>,
) -> Result<Json<AppendJournalResponse>, StatusCode> {
    let date = req.date.unwrap_or_else(|| chrono::Local::now().date_naive());
    let path = vault::journal::append_note(&cfg.vault_path, date, &req.note)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AppendJournalResponse { path }))
}
