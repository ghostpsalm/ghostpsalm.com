pub mod pushover;

use std::sync::Arc;

use anyhow::{Context, Result};
use chrono_tz::Tz;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

use crate::db::models::Task;
use crate::state::AppState;

// ── Scheduler bootstrap ────────────────────────────────────────────────────

pub async fn start_scheduler(state: AppState) -> Result<JobScheduler> {
    let tz: Tz = state
        .cfg
        .scheduler_tz
        .parse()
        .with_context(|| format!("invalid SCHEDULER_TZ: {}", state.cfg.scheduler_tz))?;

    tracing::info!(
        tz = %state.cfg.scheduler_tz,
        pushover = state.cfg.pushover_ready(),
        "starting notification scheduler"
    );

    let sched = JobScheduler::new().await?;

    // Wrap each job function in Arc so it can be cloned into the FnMut closure.
    add_job(&sched, &state.cfg.morning_brief_cron.clone(), tz, state.clone(), Arc::new(morning_brief)).await?;
    add_job(&sched, &state.cfg.lunch_review_cron.clone(), tz, state.clone(), Arc::new(lunchtime_review)).await?;
    add_job(&sched, &state.cfg.evening_errands_cron.clone(), tz, state.clone(), Arc::new(evening_errands)).await?;

    sched.start().await?;
    Ok(sched)
}

async fn add_job<F, Fut>(
    sched: &JobScheduler,
    cron: &str,
    tz: Tz,
    state: AppState,
    f: Arc<F>,
) -> Result<()>
where
    F: Fn(AppState) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<()>> + Send + 'static,
{
    let job = Job::new_async_tz(cron, tz, move |_, _| {
        let s = state.clone();
        let f = f.clone();
        Box::pin(async move {
            if let Err(e) = f(s).await {
                tracing::error!("scheduled notification failed: {e:#}");
            }
        })
    })?;
    sched.add(job).await?;
    Ok(())
}

// ── Job implementations ────────────────────────────────────────────────────

pub async fn morning_brief(state: AppState) -> Result<()> {
    let tasks = open_tasks(&state).await?;
    let total = tasks.len();
    let due_today = tasks
        .iter()
        .filter(|t| {
            t.due_date.map_or(false, |d| {
                d.date_naive() == chrono::Local::now().date_naive()
            })
        })
        .count();

    let message = format!(
        "Good morning. {total} open tasks ({due_today} due today). Start with your morning check-in."
    );

    send_notification(&state, "morning_brief", "Morning Brief", &message, Some(-1)).await
}

pub async fn lunchtime_review(state: AppState) -> Result<()> {
    let tasks = open_tasks(&state).await?;
    if tasks.is_empty() {
        return log_skipped(&state, "lunchtime_review", "no open tasks").await;
    }

    let errand_ids: std::collections::HashSet<Uuid> = tasks
        .iter()
        .filter(|t| location_contains(t, "errand") || location_contains(t, "shop"))
        .map(|t| t.id)
        .collect();
    let call_ids: std::collections::HashSet<Uuid> = tasks
        .iter()
        .filter(|t| t.person.is_some() && !errand_ids.contains(&t.id))
        .map(|t| t.id)
        .collect();
    let health_ids: std::collections::HashSet<Uuid> = tasks
        .iter()
        .filter(|t| project_contains(t, "health") && !errand_ids.contains(&t.id) && !call_ids.contains(&t.id))
        .map(|t| t.id)
        .collect();

    let mut sections: Vec<String> = Vec::new();

    let errands: Vec<_> = tasks.iter().filter(|t| errand_ids.contains(&t.id)).collect();
    let calls: Vec<_> = tasks.iter().filter(|t| call_ids.contains(&t.id)).collect();
    let health: Vec<_> = tasks.iter().filter(|t| health_ids.contains(&t.id)).collect();
    let other: Vec<_> = tasks
        .iter()
        .filter(|t| {
            !errand_ids.contains(&t.id)
                && !call_ids.contains(&t.id)
                && !health_ids.contains(&t.id)
        })
        .collect();

    if !errands.is_empty() {
        sections.push(format!("Errands ({}): {}", errands.len(), titles(&errands)));
    }
    if !calls.is_empty() {
        sections.push(format!("Calls/messages ({}): {}", calls.len(), titles(&calls)));
    }
    if !health.is_empty() {
        sections.push(format!("Health ({}): {}", health.len(), titles(&health)));
    }
    if !other.is_empty() {
        sections.push(format!("Other ({}): {}", other.len(), titles(&other)));
    }

    let message = format!("Lunchtime review — {} open\n\n{}", tasks.len(), sections.join("\n"));
    send_notification(&state, "lunchtime_review", "Lunchtime Review", &message, Some(0)).await
}

pub async fn evening_errands(state: AppState) -> Result<()> {
    let tasks = open_tasks(&state).await?;
    let errands: Vec<_> = tasks
        .iter()
        .filter(|t| {
            location_contains(t, "errand")
                || location_contains(t, "shop")
                || project_contains(t, "errand")
        })
        .collect();

    if errands.is_empty() {
        return log_skipped(&state, "evening_errands", "no errand tasks open").await;
    }

    let list = errands
        .iter()
        .map(|t| format!("- {}", t.title))
        .collect::<Vec<_>>()
        .join("\n");
    let message = format!("Evening reminder — {} errand(s) open:\n{}", errands.len(), list);

    send_notification(&state, "evening_errands", "Evening Errands", &message, Some(0)).await
}

// ── Test / manual trigger (called from API handler) ────────────────────────

pub async fn send_test(state: &AppState) -> Result<()> {
    send_notification(
        state,
        "test",
        "Ghostpsalm Test",
        "Notification delivery confirmed.",
        Some(-1),
    )
    .await
}

// ── Shared helpers ─────────────────────────────────────────────────────────

async fn open_tasks(state: &AppState) -> Result<Vec<Task>> {
    Ok(sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE status='open' ORDER BY priority ASC NULLS LAST, due_date ASC NULLS LAST",
    )
    .fetch_all(&state.db)
    .await?)
}

async fn send_notification(
    state: &AppState,
    kind: &str,
    title: &str,
    message: &str,
    priority: Option<i8>,
) -> Result<()> {
    let payload = serde_json::json!({ "title": title, "message": message });

    if !state.cfg.pushover_ready() {
        tracing::info!(kind, "pushover not configured — skipping notification");
        return log_to_db(state, kind, &payload, "skipped", None).await;
    }

    let token = state.cfg.pushover_token.as_deref().unwrap();
    let user = state.cfg.pushover_user.as_deref().unwrap();

    let result = pushover::send(
        &state.http,
        pushover::Message {
            token,
            user,
            title,
            message,
            priority,
            sound: None,
        },
    )
    .await;

    match &result {
        Ok(_) => {
            tracing::info!(kind, "notification sent via Pushover");
            log_to_db(state, kind, &payload, "sent", None).await?;
        }
        Err(e) => {
            tracing::error!(kind, error = %e, "notification failed");
            log_to_db(state, kind, &payload, "failed", Some(&e.to_string())).await?;
        }
    }

    result
}

async fn log_skipped(state: &AppState, kind: &str, reason: &str) -> Result<()> {
    tracing::debug!(kind, reason, "notification skipped");
    log_to_db(
        state,
        kind,
        &serde_json::json!({ "reason": reason }),
        "skipped",
        None,
    )
    .await
}

async fn log_to_db(
    state: &AppState,
    kind: &str,
    payload: &serde_json::Value,
    status: &str,
    error: Option<&str>,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO notification_log (id, kind, channel, payload, status, error) VALUES ($1, $2, 'pushover', $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(kind)
    .bind(payload)
    .bind(status)
    .bind(error)
    .execute(&state.db)
    .await?;
    Ok(())
}

fn location_contains(t: &Task, needle: &str) -> bool {
    t.location.as_deref().unwrap_or("").contains(needle)
}

fn project_contains(t: &Task, needle: &str) -> bool {
    t.project.as_deref().unwrap_or("").contains(needle)
}

fn titles(tasks: &[&Task]) -> String {
    let mut s: Vec<String> = tasks.iter().take(3).map(|t| t.title.clone()).collect();
    if tasks.len() > 3 {
        s.push(format!("+{}", tasks.len() - 3));
    }
    s.join(", ")
}
