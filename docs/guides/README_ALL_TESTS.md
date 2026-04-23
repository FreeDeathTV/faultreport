# Start everything

docker-compose up -d

# Unit tests (fast, no DB)

docker-compose exec backend cargo test --lib

# Integration tests (with DB)

docker-compose exec backend cargo test --test integration_test

# Benchmarks (load testing)

docker-compose exec backend cargo test --test bench_test -- --nocapture

# View logs

docker-compose logs -f backend

# Clean up

docker-compose down

---

## Local integration test notes (recommended)

The repository includes ignored integration/e2e tests that require a running Postgres and the backend migrations to be applied.

1. Bring up services (fresh):

```bash
docker compose down -v
docker compose up --build -d
```

2. Export a test DB URL for the backend tests (matches `docker-compose.yml` defaults):

```bash
export TEST_DATABASE_URL="postgresql://postgres:password@localhost:5432/faultreport"
```

3. Wait for Postgres & backend to be healthy (backend exposes `/api/healthz`):

```bash
# wait until healthy
docker compose ps
# or poll the backend health endpoint
curl http://localhost:8000/api/healthz
```

4. Run the ignored integration tests (these exercise the full DB-backed flows):

```bash
cd backend
cargo test -- --ignored --test-threads=1
```

Or run a single ignored test:

```bash
cargo test e2e_determinism -- --ignored
```

Optional environment variables useful for local/CI testing (can be set in `docker-compose.yml` or exported):

- `FIREBASE_CERTS_URL` — URL to fetch JWT certs (used by backend auth verifier)
- `FIREBASE_PROJECT_ID` — expected Firebase project id (aud validation)
- `SLACK_WEBHOOK_URL` — Slack webhook for alert testing
- `NO_EXTERNAL_CALLS` — set to `1` in CI to prevent external network calls

These tests are intentionally run with `--ignored` by default; once a test DB is available in CI you can enable them in a workflow or remove `#[ignore]` from selected tests.
