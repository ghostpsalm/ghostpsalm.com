CREATE TABLE revoked_tokens (
    jti        TEXT PRIMARY KEY,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reason     TEXT
);

CREATE INDEX idx_revoked_tokens_revoked_at ON revoked_tokens (revoked_at DESC);
