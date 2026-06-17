use axum::{extract::State, http::StatusCode, Json};
use sqlx::PgPool;

use crate::config::Config;
use crate::db::models::AuditEvent;

pub async fn recent(
    State((db, _cfg)): State<(PgPool, Config)>,
) -> Result<Json<Vec<AuditEvent>>, StatusCode> {
    let events = sqlx::query_as::<_, AuditEvent>(
        "SELECT * FROM audit_events ORDER BY created_at DESC LIMIT 50",
    )
    .fetch_all(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(events))
}
