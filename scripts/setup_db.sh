#!/usr/bin/env bash
set -euo pipefail

DB_USER="ghostpsalm"
DB_NAME="ghostpsalm"
DB_PASS="$(openssl rand -base64 24 | tr -d '/+=' | head -c 32)"

# Create role and database as postgres superuser (peer auth)
sudo -u postgres psql <<SQL
DO \$\$
BEGIN
  IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = '${DB_USER}') THEN
    CREATE ROLE ${DB_USER} LOGIN PASSWORD '${DB_PASS}';
  ELSE
    ALTER ROLE ${DB_USER} WITH PASSWORD '${DB_PASS}';
  END IF;
END
\$\$;

SELECT 'Role ready: ${DB_USER}';

DROP DATABASE IF EXISTS ${DB_NAME};
CREATE DATABASE ${DB_NAME} OWNER ${DB_USER};
SELECT 'Database ready: ${DB_NAME}';
SQL

echo ""
echo "Writing .env to $(dirname "$0")/../.env"

cat > "$(dirname "$0")/../.env" <<ENV
DATABASE_URL=postgres://${DB_USER}:${DB_PASS}@localhost:5432/${DB_NAME}
VAULT_PATH=./vault
BIND_ADDR=127.0.0.1:3000
JWT_SECRET=$(openssl rand -base64 48 | tr -d '\n')
TOKEN_TTL_SECS=3600
GHOSTPSALM_PASSPHRASE=$(openssl rand -base64 24 | tr -d '/+=' | head -c 20)
ENV

echo "Done. .env written — keep it out of git."
echo "Run: cargo sqlx migrate run"
