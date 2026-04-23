# Issue: Add integration test harness (Docker Compose + migrations)

**Priority:** High

**Summary:** Provide a reproducible integration test harness that brings up Postgres (and Redis if needed), applies migrations, and exposes `TEST_DATABASE_URL` for integration tests.

**Acceptance criteria:**

- `tests/docker-compose.test.yml` defines `postgres` service with a known password and DB
- `tests/test-harness` helper script brings up services, waits for readiness, applies migrations, and prints `TEST_DATABASE_URL`
- Integration tests can be run with `TEST_DATABASE_URL` set
- CI can opt-in to run integration tests using the harness

**Implementation steps:**

1. Add `tests/docker-compose.test.yml` (Postgres, optional Redis)
2. Add `scripts/test-harness.ps1` (Windows) and `scripts/test-harness.sh` (Linux/macOS) to start services and run migrations
3. Add test helper in Rust `tests/support/mod.rs` to load `TEST_DATABASE_URL` and create connection pool
4. Add documentation in `README_TESTING.md` with commands to run integration tests locally and in CI

**Estimated effort:** 1-2 days
