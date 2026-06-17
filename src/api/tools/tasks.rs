use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::db::models::Task;

#[derive(Debug, Deserialize)]
pub struct AddTaskRequest {
    pub title: String,
    pub priority: Option<i16>,
    pub due_date: Option<chrono::DateTime<Utc>>,
    pub project: Option<String>,
    pub person: Option<String>,
    pub location: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CompleteTaskRequest {
    pub id: Uuid,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskRequest {
    pub id: Uuid,
    pub title: Option<String>,
    pub priority: Option<i16>,
    pub due_date: Option<chrono::DateTime<Utc>>,
    pub project: Option<String>,
    pub person: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub task: Task,
}

pub async fn add(
    State((db, _cfg)): State<(PgPool, Config)>,
    Json(req): Json<AddTaskRequest>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let task = sqlx::query_as::<_, Task>(
        r#"INSERT INTO tasks (id, title, status, priority, due_date, project, person, location, source, created_at)
           VALUES ($1, $2, 'open', $3, $4, $5, $6, $7, $8, NOW())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(&req.title)
    .bind(req.priority)
    .bind(req.due_date)
    .bind(&req.project)
    .bind(&req.person)
    .bind(&req.location)
    .bind(req.source.as_deref().unwrap_or("assistant"))
    .fetch_one(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TaskResponse { task }))
}

pub async fn complete(
    State((db, _cfg)): State<(PgPool, Config)>,
    Json(req): Json<CompleteTaskRequest>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let task = sqlx::query_as::<_, Task>(
        "UPDATE tasks SET status='done', completed_at=NOW() WHERE id=$1 RETURNING *",
    )
    .bind(req.id)
    .fetch_one(&db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(TaskResponse { task }))
}

pub async fn update(
    State((db, _cfg)): State<(PgPool, Config)>,
    Json(req): Json<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let task = sqlx::query_as::<_, Task>(
        r#"UPDATE tasks SET
             title     = COALESCE($2, title),
             priority  = COALESCE($3, priority),
             due_date  = COALESCE($4, due_date),
             project   = COALESCE($5, project),
             person    = COALESCE($6, person),
             status    = COALESCE($7, status)
           WHERE id = $1 RETURNING *"#,
    )
    .bind(req.id)
    .bind(&req.title)
    .bind(req.priority)
    .bind(req.due_date)
    .bind(&req.project)
    .bind(&req.person)
    .bind(&req.status)
    .fetch_one(&db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(TaskResponse { task }))
}
