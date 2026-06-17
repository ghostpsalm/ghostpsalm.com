CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE tasks (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title         TEXT NOT NULL,
    status        TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'done', 'deferred')),
    priority      SMALLINT CHECK (priority BETWEEN 1 AND 3),
    due_date      TIMESTAMPTZ,
    project       TEXT,
    person        TEXT,
    location      TEXT,
    source        TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at  TIMESTAMPTZ
);

CREATE TABLE health_entries (
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    date             DATE NOT NULL,
    fatigue_rating   SMALLINT CHECK (fatigue_rating BETWEEN 1 AND 10),
    glucose_readings JSONB,
    sleep_hours      REAL,
    symptoms         TEXT,
    notes            TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE purchases (
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    item         TEXT NOT NULL,
    vendor       TEXT,
    status       TEXT NOT NULL DEFAULT 'want' CHECK (status IN ('want', 'ordered', 'paid', 'collected')),
    price_cents  BIGINT,
    link         TEXT,
    project      TEXT,
    ordered_at   TIMESTAMPTZ,
    paid_at      TIMESTAMPTZ,
    delivered_at TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE audit_events (
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id    TEXT,
    tool          TEXT NOT NULL,
    payload       JSONB NOT NULL DEFAULT '{}',
    files_touched TEXT[] NOT NULL DEFAULT '{}',
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tasks_status    ON tasks (status);
CREATE INDEX idx_tasks_project   ON tasks (project);
CREATE INDEX idx_tasks_person    ON tasks (person);
CREATE INDEX idx_health_date     ON health_entries (date);
CREATE INDEX idx_audit_created   ON audit_events (created_at DESC);
