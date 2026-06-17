use axum::{extract::State, http::StatusCode, Json};

use crate::db::models::Task;
use crate::state::AppState;

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Task>>, StatusCode> {
    let tasks = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE status='open' AND person IS NOT NULL ORDER BY priority ASC NULLS LAST",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tasks))
}
