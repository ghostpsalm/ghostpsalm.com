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

    // Protected routes — all require a valid EdDSA JWT.
    let protected = Router::new()
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
        .route("/auth/revoke", post(auth::revoke_token))
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
