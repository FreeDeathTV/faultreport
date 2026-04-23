# STUBS -> TODOs

Last updated: 2026-04-13

This file summarizes stubbed implementations and test placeholders discovered in the repo, what has been completed, and the concrete next steps to finish remediation and make them PR-ready.

Summary (current state)

- Alerting / Slack posting
  - Location: backend/src/modules/alert.rs
  - Status: Slack posting implemented (`post_slack_raw` / `post_slack`), guarded by `NO_EXTERNAL_CALLS`, integration sink test present
  - Remaining: Optional real-Slack test (requires test webhook) and documentation of webhook env usage

- Testing placeholders
  - Locations: backend tests and some `assert!(true)` placeholders; integration harness present under `tests/` and `tests/integration`
  - Status: Basic e2e/integration tests added; unit tests need expansion and CI configuration
  - Remaining: Replace placeholder asserts with real assertions, add CI job to run integration harness (opt-in)

- Frontend auth
  - Location: frontend/src/auth/firebase.ts
  - Status: Firebase stub present for MVP
  - Remaining: Add configurable auth client (env switch) and E2E login test

- Integration harness
  - Files: tests/docker-compose.test.yml, scripts/test-harness.sh/ps1, tests/integration/post_slack_integration.rs
  - Status: Present and usable locally
  - Remaining: Add optional CI workflow to exercise harness and run critical e2e tests

Completed work (high level)

- Implemented alert spike detection and Slack helpers (with retry/backoff)
- Added `NO_EXTERNAL_CALLS` guard to avoid external requests in CI
- Added integration test sink and a test for Slack posting
- Added scripts to apply migrations and a docker-compose test file for local integration tests

Remaining work (actionable items)

- Backend
  - [ ] Expand unit tests: replace `assert!(true)` placeholders with real DB-backed assertions
  - [ ] Add CI job to run integration tests behind a feature flag (optional, timeboxed)
  - [ ] Verify and document required env vars: `SLACK_WEBHOOK_URL`, `NO_EXTERNAL_CALLS`, `TEST_DATABASE_URL`

Note: DB-dependent e2e/integration tests have been annotated `#[ignore]` so the default
`cargo test` run won't execute them until a `TEST_DATABASE_URL` is provided and the
tests are run with `cargo test -- --ignored` (or the attribute is removed).

- Frontend
  - [ ] Replace Firebase stub with configurable client (env toggle)
  - [ ] Add a lightweight E2E test for login flow (can run in CI gating when available)

- Repo/CI
  - [ ] Create optional GitHub Actions workflow to spin up `tests/docker-compose.test.yml`, run migrations, then run integration tests (opt-in via workflow dispatch)
  - [ ] Draft PR(s) and link issue skeletons from `docs/stub_issues/` for tracking

Next recommended steps (I can take these)

1. Add a small CI workflow that runs only the critical integration tests using the existing `tests/docker-compose.test.yml` (opt-in PR).
2. Replace obvious test placeholders with real assertions (I can implement a few as examples).
3. Open a draft PR titled `chore(stubs): convert runtime stubs into TODO checklist` and include this file plus CI suggestion.

If you want, I will: implement the CI workflow (1), convert 2–3 test placeholders into real tests (2), and open the draft PR (3). Tell me which of these tasks you'd like me to do next.
