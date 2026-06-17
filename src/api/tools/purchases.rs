use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::Config;
use crate::db::models::Purchase;

#[derive(Debug, Deserialize)]
pub struct RecordPurchaseRequest {
    pub item: String,
    pub vendor: Option<String>,
    pub status: Option<String>,
    pub price_cents: Option<i64>,
    pub link: Option<String>,
    pub project: Option<String>,
}

pub async fn record(
    State((db, _cfg)): State<(PgPool, Config)>,
    Json(req): Json<RecordPurchaseRequest>,
) -> Result<Json<Purchase>, StatusCode> {
    let purchase = sqlx::query_as::<_, Purchase>(
        r#"INSERT INTO purchases (id, item, vendor, status, price_cents, link, project, created_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
           RETURNING *"#,
    )
    .bind(Uuid::new_v4())
    .bind(&req.item)
    .bind(&req.vendor)
    .bind(req.status.as_deref().unwrap_or("want"))
    .bind(req.price_cents)
    .bind(&req.link)
    .bind(&req.project)
    .fetch_one(&db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(purchase))
}
