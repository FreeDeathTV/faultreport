# Issue: Implement Alerting / Slack integration

**Priority:** High

**Summary:** Replace the current best-effort `post_slack` helper with a robust alerting implementation that supports webhook configuration, retry/backoff, and a test sink.

**Background:** `backend/src/modules/alert.rs` performs spike detection but previously had posting stubbed. A simple `post_slack` was added; this issue covers hardening it.

**Acceptance criteria:**

- Outbound Slack posting is configurable via env and config struct
- Posts use exponential backoff on failures (circuit-breaker for repeated failures)
- A test sink (local HTTP endpoint) can be used in CI to capture outgoing webhook payloads
- Alerts are logged with structured metadata

**Implementation steps:**

1. Create `AlertClient` abstraction that encapsulates HTTP client and retry logic.
2. Move `post_slack` into `AlertClient::post_alert(...)` and inject via config or from `PgPoolExt`/app state.
3. Add a small in-repo test sink server for CI (`tests/support/webhook_sink.rs`) or use httptest crate.
4. Add tests: unit tests for payload formatting, integration test posting to test sink.
5. Add configuration for circuit breaker / disable feature (`NO_EXTERNAL_CALLS` already implemented).

**Estimated effort:** 4-8 hours
