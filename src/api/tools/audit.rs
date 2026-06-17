use axum::{extract::State, http::StatusCode, Json};

use crate::db::models::AuditEvent;
use crate::state::AppState;

pub async fn recent(State(state): State<AppState>) -> Result<Json<Vec<AuditEvent>>, StatusCode> {
    let events = sqlx::query_as::<_, AuditEvent>(
        "SELECT * FROM audit_events ORDER BY created_at DESC LIMIT 50",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(events))
}
