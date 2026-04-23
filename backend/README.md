# Backend — Local development & tests

This file contains quick commands to run the backend and execute integration tests locally.

Prerequisites:

- Docker & Docker Compose
- Rust toolchain (for running tests locally)

Bring services up (from repo root):

```bash
# Fresh start
docker compose down -v
docker compose up --build -d
```

Unit tests (fast, no DB):

```bash
cd backend
cargo test --lib
```

Integration / e2e tests (require Postgres)

```bash
# export TEST_DATABASE_URL to point at the Postgres service
export TEST_DATABASE_URL="postgresql://postgres:password@localhost:5432/faultreport"

# wait for Postgres and backend to be healthy
curl http://localhost:8000/api/healthz

# run all ignored integration tests (they exercise DB-backed flows)
cargo test -- --ignored --test-threads=1

# run a single ignored test
cargo test e2e_determinism -- --ignored
```

Optional env vars useful for local/CI testing:

- `FIREBASE_CERTS_URL` — URL to fetch JWT certs (used by backend auth verifier)
- `FIREBASE_PROJECT_ID` — expected Firebase project id (aud validation)
- `SLACK_WEBHOOK_URL` — Slack webhook for alert testing
- `NO_EXTERNAL_CALLS` — set to `1` in CI to prevent external network calls

These instructions mirror `README_ALL_TESTS.md` and provide backend-specific guidance.
