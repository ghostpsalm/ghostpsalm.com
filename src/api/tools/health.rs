use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::db::models::HealthEntry;
use crate::vault;

#[derive(Debug, Deserialize)]
pub struct LogHealthRequest {
    pub date: Option<chrono::NaiveDate>,
    pub fatigue_rating: Option<i16>,
    pub glucose_readings: Option<serde_json::Value>,
    pub sleep_hours: Option<f32>,
    pub symptoms: Option<String>,
    pub notes: Option<String>,
}

pub async fn log_entry(
    State((db, cfg)): State<(PgPool, Config)>,
    Json(req): Json<LogHealthRequest>,
) -> Result<Json<HealthEntry>, StatusCode> {
    let date = req.date.unwrap_or_else(|| chrono::Local::now().date_naive());

    let entry = sqlx::query_as::<_, HealthEntry>(
        r#"INSERT INTO health_entries (id, date, fatigue_rating, glucose_readings, sleep_hours, symptoms, notes, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(date)
    .bind(req.fatigue_rating)
    .bind(&req.glucose_readings)
    .bind(req.sleep_hours)
    .bind(&req.symptoms)
    .bind(&req.notes)
    .fetch_one(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    vault::journal::append_health(&cfg.vault_path, &entry)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(entry))
}
