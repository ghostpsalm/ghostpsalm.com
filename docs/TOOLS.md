# Tool Reference

Full schema reference for every MCP tool endpoint. All protected endpoints require:

```
Authorization: Bearer <token>
Content-Type: application/json   (for POST endpoints)
```

---

## Authentication

### `POST /auth/token`

Issue a short-lived JWT. No auth required.

**Request**
```json
{
  "passphrase": "your-passphrase",
  "scope": "write"
}
```

`scope` — `"read"` or `"write"` (default `"write"`). Read tokens are rejected with `403` on any mutating endpoint.

**Response**
```json
{
  "token": "eyJ...",
  "expires_at": "2026-06-17T14:00:00Z",
  "scope": "write"
}
```

---

### `GET /auth/me`

Inspect the current token's claims. Useful for phone/assistant to check scope and expiry.

**Response**
```json
{
  "sub": "assistant",
  "scope": "write",
  "jti": "6df0b166-f821-4f36-be00-c281f0eb9bc0",
  "expires_at": "2026-06-17T14:00:00Z",
  "revoked": false
}
```

---

### `POST /auth/revoke`

Blacklist the current token. Subsequent requests with the same token return `401`.

**Response**: `204 No Content`

---

## Tasks

### `GET /api/tools/daily_review`

Returns all open tasks grouped by context bucket.

**Response**
```json
{
  "errands":       [ ...tasks... ],
  "calls_messages":[ ...tasks... ],
  "house":         [ ...tasks... ],
  "projects":      [ ...tasks... ],
  "health":        [ ...tasks... ],
  "other":         [ ...tasks... ]
}
```

Bucketing logic uses `location` and `project` fields. Tasks with a `person` field go to `calls_messages`.

---

### `POST /api/tools/add_task`

Create a task. The `title` is required; all other fields are optional.

**Request**
```json
{
  "title": "Pick up trailer from Rob",
  "priority": 1,
  "due_date": "2026-06-20T09:00:00Z",
  "project": "shed",
  "person": "Rob",
  "location": "errand",
  "source": "voice"
}
```

`priority` — `1` (high), `2` (normal), `3` (low).
`source` — `"voice"`, `"text"`, `"assistant"` (default).

**Response**: `{ "task": { ...Task... } }`

---

### `POST /api/tools/complete_task`

Mark a task done.

**Request**
```json
{
  "id": "uuid",
  "note": "optional completion note"
}
```

**Response**: `{ "task": { ...Task... } }`

---

### `POST /api/tools/update_task`

Update any field on a task. Omitted fields are unchanged (COALESCE semantics).

**Request**
```json
{
  "id": "uuid",
  "title": "Updated title",
  "priority": 2,
  "due_date": "2026-06-25T00:00:00Z",
  "project": "dragoon",
  "person": null,
  "status": "deferred"
}
```

`status` — `"open"`, `"done"`, `"deferred"`.

---

### Task object schema

```json
{
  "id": "uuid",
  "title": "string",
  "status": "open | done | deferred",
  "priority": 1,
  "due_date": "2026-06-20T09:00:00Z | null",
  "project": "string | null",
  "person": "string | null",
  "location": "string | null",
  "source": "string | null",
  "created_at": "timestamp",
  "completed_at": "timestamp | null"
}
```

---

## Health Logging

### `POST /api/tools/log_health`

Append a structured health entry. Also writes to the daily journal markdown file.

**Request**
```json
{
  "date": "2026-06-17",
  "fatigue_rating": 6,
  "glucose_readings": [5.2, 6.1, 4.9],
  "sleep_hours": 7.5,
  "symptoms": "mild headache, afternoon fatigue",
  "notes": "Took medication at 8am. Energy improved after lunch."
}
```

All fields are optional. `date` defaults to today.
`fatigue_rating` — 1 (none) to 10 (severe).

**Response**: The created `HealthEntry` record.

---

## Journal

### `POST /api/tools/append_journal`

Append a plain note to the daily Markdown journal. Creates the file if it doesn't exist.

**Request**
```json
{
  "note": "Finished bookbinding spine gluing. Let dry overnight.",
  "date": "2026-06-17"
}
```

`date` defaults to today. The note is appended under a `## HH:MM` heading.

**Response**
```json
{ "path": "./vault/journal/2026/2026-06-17.md" }
```

---

## Knowledge

### `POST /api/tools/save_knowledge`

Write a titled Markdown note into the vault. Creates the file with YAML frontmatter.

**Request**
```json
{
  "title": "Dragoon monorepo migration plan",
  "content": "## Overview\n\nMigrate all sibling repos into a single Cargo workspace...",
  "vault_path": "projects/dragoon"
}
```

`vault_path` is relative to the vault root. Defaults to `"notes"`.

**Response**
```json
{ "path": "./vault/projects/dragoon/dragoon-monorepo-migration-plan.md" }
```

---

## Contacts

### `GET /api/tools/contacts_to_message`

Return all open tasks that have a `person` field set — these represent people you need to contact.

**Response**: Array of `Task` objects.

---

## Purchases

### `POST /api/tools/record_purchase`

Track an item to buy, on order, or already collected.

**Request**
```json
{
  "item": "Bookbinding bone folder",
  "vendor": "Eckersley's",
  "status": "want",
  "price_cents": 1200,
  "link": null,
  "project": "bookbinding"
}
```

`status` — `"want"`, `"ordered"`, `"paid"`, `"collected"`.
`price_cents` — integer cents (e.g., `1200` = $12.00).

**Response**: The created `Purchase` record.

---

## Git Sync

### `POST /api/tools/sync_git`

Commit vault changes, pull with rebase, and push.

**Request**
```json
{
  "message": "assistant: afternoon sync 2026-06-17"
}
```

`message` is optional — defaults to `"assistant: sync YYYY-MM-DD HH:MM"`.

**Response**
```json
{
  "committed": true,
  "commit_hash": "43e6628",
  "files_changed": 3,
  "pulled": true,
  "pushed": true,
  "conflicts": false,
  "conflict_files": [],
  "message": "assistant: afternoon sync 2026-06-17"
}
```

**Conflict handling**: if `pulled: false` and `conflicts: true`, the rebase was aborted safely. Local commit is intact. Resolve conflicts manually and re-sync.

Auto-sync runs on `VAULT_SYNC_CRON` (default every 4 hours) without needing an explicit call.

---

## Audit

### `GET /api/tools/audit_recent`

Return the last 50 tool call records.

**Response**: Array of `AuditEvent` objects.

```json
{
  "id": "uuid",
  "session_id": "jti-of-the-token",
  "tool": "POST /api/tools/add_task",
  "payload": { "status": 200, "scope": "write" },
  "files_touched": [],
  "created_at": "timestamp"
}
```

---

## Notifications

### `GET /api/tools/notification_schedules`

Return the current scheduler configuration.

**Response**
```json
{
  "scheduler_tz": "Australia/Perth",
  "morning_brief_cron": "0 0 8 * * *",
  "lunch_review_cron": "0 30 12 * * *",
  "evening_errands_cron": "0 30 17 * * *",
  "pushover_enabled": true
}
```

---

### `GET /api/tools/notification_log`

Return the last 100 notification attempts.

**Response**: Array of `NotificationLogEntry` objects.

```json
{
  "id": "uuid",
  "kind": "lunchtime_review | evening_errands | morning_brief | test",
  "channel": "pushover",
  "payload": { "title": "Lunchtime Review", "message": "..." },
  "status": "sent | failed | skipped",
  "error": null,
  "sent_at": "timestamp"
}
```

`skipped` means Pushover is disabled or there were no tasks to report.

---

### `POST /api/tools/notify_test`

Fire a test notification immediately. Use to confirm Pushover credentials work.

**Response**
```json
{
  "status": "ok",
  "pushover_enabled": true
}
```

---

## Tool Manifest

### `GET /api/tools`

Returns the full tool list with a SHA-256 hash of the manifest. The hash changes whenever the tool surface changes — clients can cache the tool list and re-fetch only when the hash differs.

**Response**
```json
{
  "version": "0.1.0",
  "manifest_hash": "sha256:a392f30a...",
  "tools": [ ...tool descriptors... ]
}
```

---

## Scheduled Jobs

The following jobs run automatically on the configured schedule (all times local to `SCHEDULER_TZ`):

| Job | Default time | Env var | Action |
|---|---|---|---|
| Morning brief | 08:00 daily | `MORNING_BRIEF_CRON` | Total open tasks + due today via Pushover |
| Lunchtime review | 12:30 daily | `LUNCH_REVIEW_CRON` | Open tasks bucketed by context |
| Evening errands | 17:30 daily | `EVENING_ERRANDS_CRON` | Open tasks tagged as errands |
| Vault auto-sync | Every 4 hours | `VAULT_SYNC_CRON` | git add → commit → pull --rebase → push |

All cron expressions use the format: `sec min hour dom month dow`
