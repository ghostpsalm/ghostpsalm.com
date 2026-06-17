use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use crate::notifications;
use crate::state::AppState;

#[derive(Debug, Serialize, FromRow)]
pub struct NotificationLogEntry {
    pub id: Uuid,
    pub kind: String,
    pub channel: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub error: Option<String>,
    pub sent_at: DateTime<Utc>,
}

pub async fn log(State(state): State<AppState>) -> Result<Json<Vec<NotificationLogEntry>>, StatusCode> {
    let entries = sqlx::query_as::<_, NotificationLogEntry>(
        "SELECT * FROM notification_log ORDER BY sent_at DESC LIMIT 100",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(entries))
}

#[derive(Debug, Serialize)]
pub struct TestResponse {
    pub status: &'static str,
    pub pushover_enabled: bool,
}

pub async fn send_test(State(state): State<AppState>) -> Result<Json<TestResponse>, (StatusCode, Json<serde_json::Value>)> {
    notifications::send_test(&state)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    Ok(Json(TestResponse {
        status: "ok",
        pushover_enabled: state.cfg.pushover_ready(),
    }))
}

#[derive(Debug, Serialize)]
pub struct ScheduleInfo {
    pub scheduler_tz: String,
    pub morning_brief_cron: String,
    pub lunch_review_cron: String,
    pub evening_errands_cron: String,
    pub pushover_enabled: bool,
}

pub async fn schedules(State(state): State<AppState>) -> Json<ScheduleInfo> {
    Json(ScheduleInfo {
        scheduler_tz: state.cfg.scheduler_tz.clone(),
        morning_brief_cron: state.cfg.morning_brief_cron.clone(),
        lunch_review_cron: state.cfg.lunch_review_cron.clone(),
        evening_errands_cron: state.cfg.evening_errands_cron.clone(),
        pushover_enabled: state.cfg.pushover_ready(),
    })
}
