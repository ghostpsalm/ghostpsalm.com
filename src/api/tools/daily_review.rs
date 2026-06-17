use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use sqlx::PgPool;

use crate::config::Config;
use crate::db::models::Task;

#[derive(Debug, Serialize)]
pub struct DailyReview {
    pub errands: Vec<Task>,
    pub calls_messages: Vec<Task>,
    pub house: Vec<Task>,
    pub projects: Vec<Task>,
    pub health: Vec<Task>,
    pub other: Vec<Task>,
}

pub async fn handler(
    State((db, _cfg)): State<(PgPool, Config)>,
) -> Result<Json<DailyReview>, StatusCode> {
    let open_tasks = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE status='open' ORDER BY priority ASC NULLS LAST, due_date ASC NULLS LAST",
    )
    .fetch_all(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut review = DailyReview {
        errands: vec![],
        calls_messages: vec![],
        house: vec![],
        projects: vec![],
        health: vec![],
        other: vec![],
    };

    for task in open_tasks {
        match task.location.as_deref().or(task.project.as_deref()) {
            Some(l) if l.contains("errand") || l.contains("shop") => review.errands.push(task),
            Some(p) if p.contains("health") => review.health.push(task),
            Some(p) if p.contains("house") || p.contains("shed") => review.house.push(task),
            _ if task.person.is_some() => review.calls_messages.push(task),
            Some(_) => review.projects.push(task),
            None => review.other.push(task),
        }
    }

    Ok(Json(review))
}
