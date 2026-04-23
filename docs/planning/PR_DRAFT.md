Title: chore(stubs): convert runtime stubs into TODO checklist + integration harness

Body:
This draft PR consolidates a set of changes that convert several runtime stubs and placeholders into tracked TODOs and implement safe, testable wiring for alerting and integration tests.

Summary of changes in this branch:

- Implement runtime Slack posting helpers (`backend/src/modules/alert.rs`) with:
  - `post_slack` (env-guarded by `NO_EXTERNAL_CALLS`)
  - `post_slack_raw` (testable helper)
  - `AlertClient` with retry/backoff
  - In-memory circuit-breaker (env-configurable thresholds)
- Add integration test harness and test sink:
  - `tests/docker-compose.test.yml` (Postgres)
  - `scripts/test-harness.sh` / `scripts/test-harness.ps1`
  - `tests/integration/post_slack_integration.rs` (httptest sink)
  - `backend/Cargo.toml` dev-deps updated (httptest)
- Add migration helpers and test support:
  - `scripts/apply-migrations.sh` / `.ps1`
  - `tests/support/mod.rs` (applies migrations using sqlx Migrator with fallback)
- Create issue skeletons under `docs/stub_issues/` for remaining remediation work
- Add `STUBS_TODO.md` summarizing the work and tracking remaining tasks

Checklist (what this PR completes):

- [x] Add `STUBS_TODO.md` with summary
- [x] Implement Slack posting and `NO_EXTERNAL_CALLS`
- [x] Harden Slack posting with retry/backoff and circuit-breaker
- [x] Add integration harness and in-repo HTTP test sink
- [x] Add migration runner scripts and test helpers
- [x] Add issue skeletons under `docs/stub_issues/`

Follow-ups (separate PRs recommended):

- Add CI workflow to run the integration harness (opt-in job)
- Implement Redis-backed circuit-breaker for multi-instance deployments
- Replace remaining `assert!(true)` test stubs with full integration tests using `setup_test_db()`
- Create GitHub issues from `docs/stub_issues/*` and link them to this PR

Test Plan:

- `cd backend && cargo test` should pass locally (unit + integration tests adjusted to use httptest sink)
- To run DB-backed integration tests:
  - `./scripts/test-harness.sh` to bring up Postgres test DB
  - `TEST_DATABASE_URL=postgresql://postgres:password@localhost:5433/faultreport_test cargo test -- --ignored`

Notes:

- This branch creates multiple files; the workspace may not be a git repo here. If you want me to push and open the PR, provide remote info or initialize git and add a remote. Otherwise save/apply the prepared patch locally and create the PR from your machine.
