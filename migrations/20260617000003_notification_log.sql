CREATE TABLE notification_log (
    id       UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    kind     TEXT NOT NULL,                -- 'lunchtime_review' | 'evening_errands' | 'morning_brief' | 'test'
    channel  TEXT NOT NULL DEFAULT 'pushover',
    payload  JSONB NOT NULL DEFAULT '{}', -- message content sent
    status   TEXT NOT NULL CHECK (status IN ('sent', 'failed', 'skipped')),
    error    TEXT,
    sent_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_notification_log_sent_at ON notification_log (sent_at DESC);
CREATE INDEX idx_notification_log_kind    ON notification_log (kind);
