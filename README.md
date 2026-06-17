# ghostpsalm.com — Personal MCP Assistant Server

A self-hosted Rust backend that turns an AI assistant conversation into a running life-management system. Manages tasks, daily health logs, knowledge notes, purchases, contacts, and project recall — all stored in plain Markdown files backed by Git, with a PostgreSQL index for fast querying.

The system is useful without AI access: files stay human-readable and Git-recoverable at all times.

---

## Architecture

| Layer | Choice | Notes |
|---|---|---|
| Language | Rust (Axum) | Single stack for web, file I/O, concurrency |
| API surface | HTTP JSON + EdDSA JWT | MCP-compatible tool endpoints |
| Primary store | Obsidian-compatible Markdown vault | `/vault/` — human-readable source of truth |
| Structured index | PostgreSQL 18 | Tasks, health entries, purchases, audit log |
| Notifications | Pushover | Morning brief, lunchtime review, evening errands |
| Scheduler | tokio-cron-scheduler | TZ-aware, configurable cron expressions |
| Git sync | Shell-out with autostash | Pull-rebase-push pipeline, conflict-safe |
| Auth | EdDSA / Ed25519 JWT | Algorithm-pinned, scoped (read/write), revocable |
| TLS | axum-server + rustls | Optional; Tailscale recommended for remote access |

---

## Build Phases

| Phase | Scope | Status |
|---|---|---|
| 0 — Files | Vault layout and Markdown conventions | Done |
| 1 — Local service | Rust API reads/writes tasks, journal, health | Done |
| 2 — MCP tools | Typed tool endpoints, describe_tools, audit logs | Done |
| 3 — Notifications | Pushover reminders for morning, lunch, evening | Done |
| 4 — Git sync | Auto-commit/pull/push with conflict protection | Done |
| 5 — Secure remote | Rate limiting, scoped tokens, TLS, Tailscale | Done |
| 6 — Integrations | Email read/draft, external knowledge bases | Planned |

---

## Vault Layout

```
vault/
├── journal/
│   └── YYYY/
│       └── YYYY-MM-DD.md        # daily log — appended by assistant
├── tasks/
│   └── inbox.md                 # raw task capture
├── projects/
│   └── <project-name>/
│       └── index.md
├── people/
│   └── <name>.md
└── purchases/
    └── <item-or-project>.md
```

All files are plain Markdown. The service never requires them to exist — it creates paths as needed.

---

## API Tools

See [docs/TOOLS.md](docs/TOOLS.md) for the full reference.

**Authentication**

| Endpoint | Method | Description |
|---|---|---|
| `/auth/token` | POST | Issue a JWT (scope: `read` or `write`) |
| `/auth/me` | GET | Inspect current token claims |
| `/auth/revoke` | POST | Blacklist the current token's jti |

**Tool endpoints** — all require `Authorization: Bearer <token>`

| Tool | Method | Scope | Description |
|---|---|---|---|
| `/api/tools` | GET | read | Tool manifest + SHA-256 hash |
| `/api/tools/daily_review` | GET | read | Open tasks grouped by context |
| `/api/tools/add_task` | POST | write | Create a task |
| `/api/tools/complete_task` | POST | write | Mark task done |
| `/api/tools/update_task` | POST | write | Change priority, due date, tags |
| `/api/tools/log_health` | POST | write | Append structured health entry |
| `/api/tools/append_journal` | POST | write | Append note to daily log |
| `/api/tools/save_knowledge` | POST | write | Write a note into the vault |
| `/api/tools/contacts_to_message` | GET | read | Open tasks with a person assigned |
| `/api/tools/record_purchase` | POST | write | Track item, vendor, status |
| `/api/tools/sync_git` | POST | write | Commit vault + pull/push |
| `/api/tools/audit_recent` | GET | read | Last 50 tool call records |
| `/api/tools/notification_log` | GET | read | Last 100 notification attempts |
| `/api/tools/notification_schedules` | GET | read | Current cron schedule config |
| `/api/tools/notify_test` | POST | write | Send a test Pushover notification |

---

## Security Model

- **JWT algorithm**: EdDSA / Ed25519 — pinned; any other `alg` claim is rejected outright.
- **Scoped tokens**: `read` tokens are rejected with 403 on any mutating endpoint.
- **Revocation**: jti blacklist in Postgres, checked on every request.
- **Rate limiting**: `AUTH_RATE_LIMIT_RPM` (default 10/min) on `/auth/token`.
- **Audit trail**: every protected call logged with session jti and response status.
- **Manifest hash**: SHA-256 of the tool list in `GET /api/tools` — changes alert clients.
- **TLS**: optional via cert files; Tailscale (WireGuard) recommended for remote access.
- **Passphrase**: constant-time comparison to prevent timing oracles.
- **Git commits**: granular and attributable — never empty, always vault-scoped.

---

## Getting Started

### Prerequisites

- Rust (stable, 2024 edition)
- PostgreSQL 18
- `openssl` (for key generation)

### Setup

```bash
# 1. Generate Ed25519 signing keys
bash scripts/gen_keys.sh

# 2. Create the database and .env
bash scripts/setup_db.sh      # requires sudo for postgres peer auth

# 3. Run migrations
cargo sqlx migrate run

# 4. Start the service
cargo run
```

### Configuration (`.env`)

```env
DATABASE_URL=postgres://ghostpsalm:password@localhost/ghostpsalm
VAULT_PATH=./vault
BIND_ADDR=127.0.0.1:3000
GHOSTPSALM_PASSPHRASE=your-long-passphrase

PRIVATE_KEY_PATH=./keys/private.pem
PUBLIC_KEY_PATH=./keys/public.pem

# Notifications (optional)
PUSHOVER_ENABLED=false
PUSHOVER_TOKEN=
PUSHOVER_USER=
SCHEDULER_TZ=Australia/Perth

# Remote access (optional — set both to enable HTTPS)
# TLS_CERT_PATH=/var/lib/tailscale/certs/hostname.crt
# TLS_KEY_PATH=/var/lib/tailscale/certs/hostname.key
```

See `.env.example` for all options.

### Remote Access via Tailscale

```bash
bash scripts/setup_tailscale.sh
# Then: sudo tailscale cert <hostname>
# Update BIND_ADDR and TLS_CERT_PATH / TLS_KEY_PATH in .env
```

### Issuing Tokens

```bash
# Write token (full access)
curl -s -X POST http://localhost:3000/auth/token \
  -H "Content-Type: application/json" \
  -d '{"passphrase":"your-passphrase","scope":"write"}'

# Read-only token (for dashboards, phone review)
curl -s -X POST http://localhost:3000/auth/token \
  -H "Content-Type: application/json" \
  -d '{"passphrase":"your-passphrase","scope":"read"}'
```

---

## Database Schema

| Table | Purpose |
|---|---|
| `tasks` | Task index — status, priority, due date, project, person |
| `health_entries` | Daily health log — fatigue, glucose, sleep, symptoms |
| `purchases` | Purchase tracking — vendor, status, price, ordered/paid/collected |
| `audit_events` | Every tool call — session jti, tool name, status |
| `revoked_tokens` | JWT jti blacklist |
| `notification_log` | Notification history — kind, status, payload |

---

## Projects Tracked

From the original brief — active in vault:

- **Dragoon** — Rust monorepo / brothers' project
- **Shed / house** — Bunnings deliveries, mould, shower base, oven/cooktop
- **Bookbinding** — tools and setup
- **Personal MCP** — this system
- **Health** — ME follow-up, fatigue and glucose logging
