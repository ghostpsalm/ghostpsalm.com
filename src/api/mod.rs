pub mod auth;
pub mod tools;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

use crate::config::Config;

pub fn router(db: PgPool, cfg: Config) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/tools", get(tools::describe_tools))
        .route("/api/tools/daily_review", get(tools::daily_review::handler))
        .route("/api/tools/add_task", post(tools::tasks::add))
        .route("/api/tools/complete_task", post(tools::tasks::complete))
        .route("/api/tools/update_task", post(tools::tasks::update))
        .route("/api/tools/log_health", post(tools::health::log_entry))
        .route("/api/tools/append_journal", post(tools::journal::append))
        .route("/api/tools/save_knowledge", post(tools::knowledge::save))
        .route("/api/tools/contacts_to_message", get(tools::contacts::list))
        .route("/api/tools/record_purchase", post(tools::purchases::record))
        .route("/api/tools/sync_git", post(tools::git_sync::sync))
        .route("/api/tools/audit_recent", get(tools::audit::recent))
        .route("/auth/token", post(auth::issue_token))
        .layer(TraceLayer::new_for_http())
        .with_state((db, cfg))
}
