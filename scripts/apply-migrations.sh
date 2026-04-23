#!/usr/bin/env bash
set -euo pipefail

if [ -z "${TEST_DATABASE_URL:-}" ]; then
  echo "ERROR: TEST_DATABASE_URL must be set" >&2
  exit 1
fi

MIGRATIONS_DIR="backend/migrations"
if [ ! -d "$MIGRATIONS_DIR" ]; then
  echo "ERROR: migrations directory not found: $MIGRATIONS_DIR" >&2
  exit 1
fi

echo "Applying migrations from $MIGRATIONS_DIR to $TEST_DATABASE_URL"

for sql in $(ls "$MIGRATIONS_DIR"/*.sql | sort); do
  echo "Applying $sql"
  psql "$TEST_DATABASE_URL" -f "$sql"
done

echo "Migrations applied."