# Issue: Replace test stubs with real tests

**Priority:** High

**Summary:** Replace unit/integration test placeholders (`assert!(true)`) with meaningful tests for `alert`, `persist`, `auth`, and storage modules.

**Background:** Several tests are stubbed and ignored. Real tests are required to ensure regressions are caught.

**Acceptance criteria:**

- Key modules have unit tests covering logic paths (hashing, normalization, rate-limit checks)
- Integration tests run against a transient Postgres instance (Docker Compose or test harness)
- CI runs unit tests and optionally runs integration tests behind a flag

**Implementation steps:**

1. Add Docker Compose service for `test-db` (Postgres) in `tests/docker-compose.test.yml` or use `testcontainers` crate.
2. Add test helper to run migrations against `TEST_DATABASE_URL` and provide a connection pool.
3. Replace `assert!(true)` with real assertions and add `#[ignore]` only where infra is not available.
4. Add CI job matrix for `unit` and `integration` (integration runs only with DB available).

**Estimated effort:** 1-2 days
