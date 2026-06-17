use axum::{extract::State, http::StatusCode, Json};
use sqlx::PgPool;

use crate::config::Config;
use crate::db::models::Task;

pub async fn list(
    State((db, _cfg)): State<(PgPool, Config)>,
) -> Result<Json<Vec<Task>>, StatusCode> {
    let tasks = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE status='open' AND person IS NOT NULL ORDER BY priority ASC NULLS LAST",
    )
    .fetch_all(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tasks))
}
