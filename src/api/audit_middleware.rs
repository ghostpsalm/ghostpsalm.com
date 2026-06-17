use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::api::auth::Claims;
use crate::state::AppState;

/// Logs every protected tool call to audit_events.
/// Runs after require_auth (so Claims are in extensions) and after the handler
/// (so we capture the response status). DB write is fire-and-forget.
pub async fn log_tool_call(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let session_id = req.extensions().get::<Claims>().map(|c| c.jti.clone());
    let scope = req.extensions().get::<Claims>().map(|c| c.scope.clone());
    let tool = req.uri().path().to_string();
    let method = req.method().to_string();

    let resp = next.run(req).await;
    let status = resp.status().as_u16() as i32;

    let db = state.db.clone();
    tokio::spawn(async move {
        let _ = sqlx::query(
            "INSERT INTO audit_events (id, session_id, tool, payload, files_touched)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(session_id.as_deref())
        .bind(format!("{} {}", method, tool))
        .bind(serde_json::json!({
            "status": status,
            "scope": scope,
        }))
        .bind(&[] as &[String])
        .execute(&db)
        .await;
    });

    resp
}
