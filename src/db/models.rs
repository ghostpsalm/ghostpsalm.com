use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, PartialEq)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub status: String,        // open | done | deferred
    pub priority: Option<i16>, // 1 high … 3 low
    pub due_date: Option<DateTime<Utc>>,
    pub project: Option<String>,
    pub person: Option<String>,
    pub location: Option<String>,
    pub source: Option<String>, // voice | text | assistant
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct HealthEntry {
    pub id: Uuid,
    pub date: chrono::NaiveDate,
    pub fatigue_rating: Option<i16>, // 1–10
    pub glucose_readings: Option<serde_json::Value>,
    pub sleep_hours: Option<f32>,
    pub symptoms: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Purchase {
    pub id: Uuid,
    pub item: String,
    pub vendor: Option<String>,
    pub status: String, // want | ordered | paid | collected
    pub price_cents: Option<i64>,
    pub link: Option<String>,
    pub project: Option<String>,
    pub ordered_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditEvent {
    pub id: Uuid,
    pub session_id: Option<String>,
    pub tool: String,
    pub payload: serde_json::Value,
    pub files_touched: Vec<String>,
    pub created_at: DateTime<Utc>,
}
