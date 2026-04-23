#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="tests/docker-compose.test.yml"

echo "Starting test services..."
docker-compose -f "$COMPOSE_FILE" up -d --build

echo "Waiting for Postgres to be ready..."
RETRY=0
until docker exec $(docker ps -qf "ancestor=postgres:15") pg_isready -U postgres -d faultreport_test >/dev/null 2>&1; do
  sleep 1
  RETRY=$((RETRY+1))
  if [ $RETRY -gt 60 ]; then
    echo "Postgres did not become ready in time." >&2
    exit 1
  fi
done

# Print TEST_DATABASE_URL for consumers
echo "TEST_DATABASE_URL=postgresql://postgres:password@localhost:5433/faultreport_test"

echo "Run tests with: TEST_DATABASE_URL=postgresql://postgres:password@localhost:5433/faultreport_test cargo test -- --ignored"
