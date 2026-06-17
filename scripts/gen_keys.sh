#!/usr/bin/env bash
# Regenerate the Ed25519 signing key pair.
# Existing tokens become invalid after regeneration.
set -euo pipefail

DIR="$(cd "$(dirname "$0")/.." && pwd)/keys"
mkdir -p "$DIR"
chmod 700 "$DIR"

openssl genpkey -algorithm ed25519 -out "$DIR/private.pem"
chmod 600 "$DIR/private.pem"
openssl pkey -in "$DIR/private.pem" -pubout -out "$DIR/public.pem"

echo "Ed25519 key pair written to $DIR"
echo "  private.pem — keep secret, never commit"
echo "  public.pem  — safe to distribute"
