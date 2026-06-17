pub mod audit;
pub mod contacts;
pub mod daily_review;
pub mod git_sync;
pub mod health;
pub mod journal;
pub mod knowledge;
pub mod purchases;
pub mod tasks;

use axum::Json;
use serde_json::{json, Value};

/// Returns the tool manifest — schema, purpose, and version.
pub async fn describe_tools() -> Json<Value> {
    Json(json!({
        "version": "0.1.0",
        "tools": [
            { "name": "describe_tools",        "method": "GET",  "path": "/api/tools",                    "scope": "read"  },
            { "name": "get_daily_review",      "method": "GET",  "path": "/api/tools/daily_review",       "scope": "read"  },
            { "name": "add_task",              "method": "POST", "path": "/api/tools/add_task",           "scope": "write" },
            { "name": "complete_task",         "method": "POST", "path": "/api/tools/complete_task",      "scope": "write" },
            { "name": "update_task",           "method": "POST", "path": "/api/tools/update_task",        "scope": "write" },
            { "name": "log_health_entry",      "method": "POST", "path": "/api/tools/log_health",         "scope": "write" },
            { "name": "append_journal_note",   "method": "POST", "path": "/api/tools/append_journal",     "scope": "write" },
            { "name": "save_knowledge_note",   "method": "POST", "path": "/api/tools/save_knowledge",     "scope": "write" },
            { "name": "list_contacts_to_message", "method": "GET", "path": "/api/tools/contacts_to_message", "scope": "read" },
            { "name": "record_purchase",       "method": "POST", "path": "/api/tools/record_purchase",   "scope": "write" },
            { "name": "sync_git",              "method": "POST", "path": "/api/tools/sync_git",           "scope": "write" },
            { "name": "audit_recent_actions",  "method": "GET",  "path": "/api/tools/audit_recent",       "scope": "read"  },
        ]
    }))
}
