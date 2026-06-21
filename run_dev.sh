#!/bin/bash
set -a
DATABASE_URL="postgres://ghostpsalm:Vkqa3XFK0BTdVt938bYzUOqXN1R4Gl4j@localhost:5432/ghostpsalm"
VAULT_PATH="./vault"
BIND_ADDR="127.0.0.1:8024"
TOKEN_TTL_SECS=7776000
GHOSTPSALM_PASSPHRASE="Let me hear in the morning of your steadfast love, for in you I trust."
PRIVATE_KEY_PATH="./keys/private.pem"
PUBLIC_KEY_PATH="./keys/public.pem"
JWT_ISSUER="ghostpsalm"
JWT_AUDIENCE="ghostpsalm-api"
AUTH_RATE_LIMIT_RPM=10
PUSHOVER_ENABLED=false
PUSHOVER_TOKEN=""
PUSHOVER_USER="uu1na6bdvt2gw1ay7ujoatasb2vhqe"
SCHEDULER_TZ="Australia/Melbourne"
LUNCH_REVIEW_CRON="0 30 12 * * *"
EVENING_ERRANDS_CRON="0 30 17 * * *"
MORNING_BRIEF_CRON="0 0 8 * * *"
VAULT_SYNC_ENABLED=true
VAULT_SYNC_CRON="0 0 */4 * * *"
set +a
exec ./target/release/ghostpsalm
