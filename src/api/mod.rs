pub mod audit_middleware;
pub mod auth;
pub mod middleware;
pub mod tools;

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    // Public routes — no auth required.
    let public = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/auth/token", post(auth::issue_token));

    // Protected routes — JWT required; every call is audit-logged.
    let protected = Router::new()
        // Tool manifest
        .route("/api/tools",                       get(tools::describe_tools))
        // Tasks
        .route("/api/tools/daily_review",          get(tools::daily_review::handler))
        .route("/api/tools/add_task",              post(tools::tasks::add))
        .route("/api/tools/complete_task",         post(tools::tasks::complete))
        .route("/api/tools/update_task",           post(tools::tasks::update))
        // Health & journal
        .route("/api/tools/log_health",            post(tools::health::log_entry))
        .route("/api/tools/append_journal",        post(tools::journal::append))
        // Knowledge
        .route("/api/tools/save_knowledge",        post(tools::knowledge::save))
        // People & purchases
        .route("/api/tools/contacts_to_message",   get(tools::contacts::list))
        .route("/api/tools/record_purchase",       post(tools::purchases::record))
        // Sync & audit
        .route("/api/tools/sync_git",              post(tools::git_sync::sync))
        .route("/api/tools/audit_recent",          get(tools::audit::recent))
        // Notifications
        .route("/api/tools/notification_log",      get(tools::notifications::log))
        .route("/api/tools/notification_schedules",get(tools::notifications::schedules))
        .route("/api/tools/notify_test",           post(tools::notifications::send_test))
        // Auth management
        .route("/auth/me",                         get(auth::me))
        .route("/auth/revoke",                     post(auth::revoke_token))
        // Audit logging runs after auth (so claims are in extensions).
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            audit_middleware::log_tool_call,
        ))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::require_auth,
        ));

    Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
