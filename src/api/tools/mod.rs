pub mod audit;
pub mod contacts;
pub mod daily_review;
pub mod git_sync;
pub mod health;
pub mod journal;
pub mod knowledge;
pub mod notifications;
pub mod purchases;
pub mod tasks;

use axum::Json;
use sha2::{Digest, Sha256};
use serde_json::{json, Value};

/// The ordered tool list — kept here as the canonical source.
/// Changing this list changes the manifest hash, alerting clients to re-fetch.
fn tool_list() -> Value {
    json!([
        { "name": "describe_tools",            "method": "GET",  "path": "/api/tools",                        "scope": "read"  },
        { "name": "get_daily_review",          "method": "GET",  "path": "/api/tools/daily_review",           "scope": "read"  },
        { "name": "add_task",                  "method": "POST", "path": "/api/tools/add_task",               "scope": "write" },
        { "name": "complete_task",             "method": "POST", "path": "/api/tools/complete_task",          "scope": "write" },
        { "name": "update_task",               "method": "POST", "path": "/api/tools/update_task",            "scope": "write" },
        { "name": "log_health_entry",          "method": "POST", "path": "/api/tools/log_health",             "scope": "write" },
        { "name": "append_journal_note",       "method": "POST", "path": "/api/tools/append_journal",         "scope": "write" },
        { "name": "save_knowledge_note",       "method": "POST", "path": "/api/tools/save_knowledge",         "scope": "write" },
        { "name": "list_contacts_to_message",  "method": "GET",  "path": "/api/tools/contacts_to_message",    "scope": "read"  },
        { "name": "record_purchase",           "method": "POST", "path": "/api/tools/record_purchase",        "scope": "write" },
        { "name": "sync_git",                  "method": "POST", "path": "/api/tools/sync_git",               "scope": "write" },
        { "name": "audit_recent_actions",      "method": "GET",  "path": "/api/tools/audit_recent",           "scope": "read"  },
        { "name": "notification_log",          "method": "GET",  "path": "/api/tools/notification_log",        "scope": "read"  },
        { "name": "notification_schedules",    "method": "GET",  "path": "/api/tools/notification_schedules",  "scope": "read"  },
        { "name": "send_test_notification",    "method": "POST", "path": "/api/tools/notify_test",            "scope": "write" },
    ])
}

pub async fn describe_tools() -> Json<Value> {
    let tools = tool_list();
    let canonical = serde_json::to_string(&tools).unwrap_or_default();
    let hash = format!("sha256:{:x}", Sha256::digest(canonical.as_bytes()));

    Json(json!({
        "version": "0.1.0",
        "manifest_hash": hash,
        "tools": tools,
    }))
}
