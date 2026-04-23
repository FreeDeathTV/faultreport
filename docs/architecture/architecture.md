# FaultReport Architecture

**Constitutional Document for Engineers**

If the building burns down and only this document survives, you can rebuild FaultReport exactly as it was.

---

## Table of Contents

1. Core Principles
2. Tech Stack (Immutable)
3. Sovereign Modules (Architecture)
4. Deterministic Rules
   4.1 Error Hashing (Deterministic Deduplication)
   4.2 Ledger Immutability
   4.3 Error Deduplication
   4.4 Why Determinism Matters
   4.5 ### IMMUTABLE HASH ALGORITHM (Constitutional)
5. Data Flow (Golden Path)
6. Module APIs (Contracts)
7. Non-Negotiable Rules
8. Testing Requirements
9. Deployment Rules
10. Recovery Procedure

---

## 1. Core Principles

### 1.1 Sovereignty

- **Every module is isolated.**
- Modules never call each other.
- All communication flows through FaultReport Orchestrator.
- No shared mutable state.
- No cross-module dependencies.

### 1.2 Determinism

- **Same input always produces same output.**
- No randomness in error normalization.
- No timestamps in hash computation.
- No nondeterministic ordering.
- Error deduplication is byte-exact.

### 1.3 Auditability

- **Every error is immutable once stored.**
- Ledger is append-only.
- Every operation is logged.
- No deletions. No mutations.
- Full audit trail reconstructable from ledger.

### 1.4 Simplicity

- **One job per module.**
- No magic.
- No hidden state.
- No background threads.
- Explicit is better than implicit.

---

## 2. Tech Stack (Immutable)

This is the stack. Do not deviate. Do not "improve it later."

### 2.1 Backend: Rust (Tokio async runtime)

**Why:**

- Memory safety (no segfaults in production)
- Deterministic (no GC pauses affecting timing)
- Type system enforces correctness
- Built-in for AI agents to work with (clear types)
- Same stack as UNFORGIVABLE (known patterns)

**Required Crates:**

```toml
[dependencies]
# Web server
actix-web = "4.x"
tokio = { version = "1.x", features = ["full"] }

# Database
sqlx = { version = "0.7.x", features = ["postgres", "runtime-tokio-native-tls", "uuid", "chrono"] }
uuid = { version = "1.x", features = ["v4", "serde"] }
chrono = { version = "0.4.x", features = ["serde"] }

# Serialization
serde = { version = "1.x", features = ["derive"] }
serde_json = "1.x"

# Crypto
sha2 = "0.10.x"
hex = "0.4.x"

# Environment
dotenv = "0.15.x"

# Logging
tracing = "0.1.x"
tracing-subscriber = "0.3.x"

# Error handling
anyhow = "1.x"
thiserror = "1.x"

# Testing
proptest = "1.x"
```

**No alternatives.** These crates are stable, production-hardened, and well-maintained.

### 2.2 Database: PostgreSQL 15+

**Why:**

- ACID guarantees
- WAL for durability
- jsonb for error context
- Full-text search (future)
- Deterministic ordering (via sequence)

**Required:**

- PostgreSQL 15+
- Connection pooling (via sqlx)
- No migrations via ORM (manual SQL files only)

**Schema Rules:**

- All tables have `created_at` (immutable)
- All tables are append-only (no UPDATE)
- All deletes are logical (soft delete via status)
- Hash column on errors table (for deduplication)
- Indexed on (hash, project_id) for fast lookups

### 2.3 Frontend: React 18 + TypeScript

**Why:**

- Simple, no magic
- TypeScript enforces correctness
- No framework overhead (not Next.js, not Remix)
- Fast to iterate

**Required Packages:**

```json
{
  "react": "18.x",
  "react-dom": "18.x",
  "typescript": "5.x",
  "vite": "5.x",
  "tailwindcss": "3.x"
}
```

**No alternatives.** No Vue. No Svelte. No Next.js.

### 2.4 Styling: Tailwind CSS

**Why:**

- Utility-first, no CSS to maintain
- Fast to prototype
- Works with Vite

**Rules:**

- No custom CSS beyond Tailwind
- Use only default colors
- No pre-designed component libraries
- Build: `tailwind -i input.css -o output.css`

### 2.5 Hosting: Railway

**Why:**

- Docker-native deployment
- Environment variables built-in
- PostgreSQL managed option
- Zero configuration needed for Rust apps

**Rules:**

- All services run in Docker
- One Docker image per module
- Environment variables only (no config files)
- Health checks mandatory

### 2.6 Payments: Stripe

**Why:**

- Industry standard
- Deterministic (no surprises)
- Webhooks for subscription events

**Implementation:**

- Stripe Billing API (not Payment Links)
- Webhook validation mandatory
- All payment state in PostgreSQL (Stripe is source of truth for billing, DB is source of truth for access)

### 2.7 Authentication: Firebase Auth

**Why:**

- Zero infrastructure
- Email + Google OAuth built-in
- Tokens are standard JWT
- No password management

**Rules:**

- Use Firebase Admin SDK on backend
- Verify tokens on every request
- Store user_id in PostgreSQL
- No custom auth logic

---

## 3. Sovereign Modules (Architecture)

### 3.1 Module Isolation Rules

**CRITICAL:** Modules never call each other. Period.

```
                    [FaultReport Orchestrator]
                             |
              |______________|______________|______________|
              |              |              |              |
        [ErrorCapture]  [Storage]      [Alert]        [Dashboard]
```

No arrows between modules. All communication flows through Orchestrator.

### 3.2 Module List

#### **Module A: Error Capture**

**Responsibility:** Validate incoming error data, normalize it, compute hash.

**Input:** HTTP POST from SDK with error payload

```json
{
  "message": "Cannot read property 'x' of undefined",
  "stack": "at line 42 in app.js",
  "context": {
    "url": "https://example.com/page",
    "browser": "Chrome 120",
    "user_id": "user123"
  }
}
```

**Output:** Normalized error object

```json
{
  "id": "uuid",
  "project_id": "uuid",
  "message": "Cannot read property 'x' of undefined",
  "hash": "abc123def456",
  "stack": "...",
  "context": {...},
  "created_at": "2024-01-15T10:30:00Z"
}
```

**Rules:**

- Remove sensitive data (passwords, tokens, API keys) before hashing
- Hash must be deterministic (same error always = same hash)
- No timestamps in hash (same error at different times = same hash)
- Validate every field (reject invalid JSON)
- Return HTTP 400 on invalid input
- Return HTTP 201 on success

**API Endpoint:**

```
POST /api/projects/:projectId/errors
Authorization: Bearer frp_... (project API key)
Content-Type: application/json

{error payload}

1. Extract Bearer key, verify vs projects.api_key_hash (SHA256), check !revoked
2. Get (project_id, user_id) from verify_api_key()
3. Override URL param projectId with verified project_id

Response: 201 Created
{
  "id": "...",
  "hash": "..."
}
```

**Tests Required:**

- Same error input → same hash every time
- Different errors → different hashes
- Sensitive data removal works
- Invalid JSON rejected
- Missing fields rejected
- Invalid API key → 401
- Revoked key → 401
- Wrong project ownership → 403

---

#### **Module B: Storage**

**Responsibility:** Persist errors to PostgreSQL, deduplicate, maintain ledger.

**Input:** Normalized error from Module A

**Output:** Stored error record (or update to existing if hash matches)

**Rules:**

- **Append-only ledger:** New error always creates new ledger entry
- **Deduplication:** If hash exists for same project, increment count instead of duplicate
- **Atomic writes:** All or nothing (use transactions)
- **No mutations:** Once stored, never change
- **Soft deletes:** Set `status='archived'` instead of DELETE

**Database Schema:**

```sql
CREATE TABLE errors (
  id UUID PRIMARY KEY,
  project_id UUID NOT NULL,
  hash VARCHAR(64) NOT NULL,
  message TEXT NOT NULL,
  stack TEXT,
  context JSONB,
  count INTEGER DEFAULT 1,
  first_seen_at TIMESTAMP NOT NULL,
  last_seen_at TIMESTAMP NOT NULL,
  status VARCHAR(20) DEFAULT 'active',
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_errors_project_hash ON errors(project_id, hash);
CREATE INDEX idx_errors_project_status ON errors(project_id, status);

CREATE TABLE ledger (
  id BIGSERIAL PRIMARY KEY,
  project_id UUID NOT NULL,
  error_id UUID NOT NULL,
  event_type VARCHAR(50) NOT NULL,
  data JSONB,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ledger_project ON ledger(project_id);
```

**API Endpoint:**

```
POST /storage/persist
Content-Type: application/json

{normalized error}

Response: 200 OK
{
  "stored_id": "...",
  "was_duplicate": true/false,
  "count": 5
}
```

**Tests Required:**

- Same error persists as deduplicated
- Different errors create separate records
- Ledger is append-only
- No mutations occur
- Transactions work correctly

---

#### **Module C: Alert**

**Responsibility:** Monitor error spike patterns, post to Slack.

**Input:** Error count change notification from Storage

**Output:** Slack message to configured webhook

**Rules:**

- **Spike detection:** If error count in last 5 minutes > 10, alert
- **Deduplication Window:** 5 minutes per hash (store last_alert_at in alert_dedup table). If alerted <5min ago, skip.
- **No duplicate alerts:** Check window before posting
- **Async queue:** Use background job queue (Tokio spawn)
- **Failure-tolerant:** If Slack is down, log and retry later

**Alert Rules (Config-Driven):**

```json
{
  "project_id": "uuid",
  "rules": [
    {
      "name": "error_spike",
      "condition": "count > 10 in last 5 minutes",
      "action": "post_to_slack",
      "slack_webhook": "https://hooks.slack.com/..."
    }
  ]
}
```

**API Endpoint:**

```
POST /alerts/trigger
Content-Type: application/json

{
  "project_id": "uuid",
  "error_id": "uuid",
  "count": 15
}

Response: 200 OK
{
  "triggered": true,
  "rule": "error_spike"
}
```

**Tests Required:**

- Spike detection logic correct
- Slack messages formatted correctly
- Duplicate alerts prevented
- Failure handling works

---

#### **Module D: Dashboard**

**Responsibility:** Serve React frontend, provide API for dashboard data.

**Input:** HTTP GET requests from browser

**Output:** JSON API responses + static HTML/JS

**Rules:**

- **No server-side rendering:** Static HTML + React hydration
- **API separates data:** Dashboard API at `/api/dashboard/*`
- **Authentication required:** Every endpoint must validate Firebase token
- **Pagination:** Return max 100 errors per request
- **Caching:** Aggressively cache error lists (5-min TTL)

**API Endpoints:**

```
GET /api/dashboard/projects/:projectId/errors
Authorization: Bearer <firebaseToken>

Response: 200 OK
{
  "errors": [
    {
      "id": "...",
      "message": "...",
      "count": 5,
      "first_seen_at": "...",
      "last_seen_at": "..."
    }
  ],
  "total": 42,
  "page": 1,
  "per_page": 20
}
```

```
GET /api/dashboard/projects/:projectId/errors/:errorId
Authorization: Bearer <firebaseToken>

Response: 200 OK
{
  "id": "...",
  "message": "...",
  "stack": "...",
  "context": {...},
  "count": 5,
  "occurrences": [
    {
      "timestamp": "...",
      "context": {...}
    }
  ]
}
```

**React Components (Minimal):**

- `ErrorList.tsx` — Table of errors
- `ErrorDetail.tsx` — Single error detail view
- `Dashboard.tsx` — Main layout
- `Login.tsx` — Firebase auth

**Tests Required:**

- Authentication enforced
- Pagination works
- Caching works
- Data formatted correctly

---

### 3.3 FaultReport Orchestrator

**Responsibility:** Route requests to modules, coordinate persistence, manage state.

**Rules:**

- **Synchronous orchestration:** Requests block until response ready
- **No business logic:** Just routing and coordination
- **State in PostgreSQL:** Orchestrator holds no state
- **Clear sequence:** Every operation follows defined path

**Request Flow:**

```
1. HTTP request arrives
2. Orchestrator receives
3. Route to appropriate module (based on path)
4. Module processes
5. Orchestrator coordinates response
6. Return to client
```

**Example: Error Submission Flow**

```
1. SDK sends POST /api/projects/:projectId/errors
2. Orchestrator routes to Module A (ErrorCapture)
3. Module A validates, normalizes, computes hash → Returns normalized error
4. Orchestrator sends to Module B (Storage)
5. Module B persists, checks for spike → Returns stored record + spike flag
6. If spike: Orchestrator triggers Module C (Alert)
7. Module C posts to Slack → Returns alert status
8. Orchestrator returns final response to SDK
```

**Code Pattern (Rust):**

```rust
pub async fn handle_error(
    req: HttpRequest,
    body: web::Json<RawError>,
) -> Result<HttpResponse> {
    // 1. Route
    let project_id = extract_project_id(&req);

    // 2. Module A: Capture
    let normalized = error_capture::normalize(&body)?;
    let hash = error_capture::hash(&normalized)?;

    // 3. Module B: Storage
    let stored = storage::persist(project_id, normalized.clone()).await?;
    let spike_detected = storage::check_spike(project_id, &hash).await?;

    // 4. Module C: Alert (if spike)
    if spike_detected {
        alert::trigger(project_id, &hash).await.ok(); // Fire-and-forget
    }

    // 5. Return
    Ok(HttpResponse::Created().json(json!({
        "id": stored.id,
        "hash": stored.hash,
    })))
}
```

---

## 4. Deterministic Rules

### 4.1 Error Hashing (Deterministic Deduplication)

**Algorithm:**

```
1. Take error message
2. Take stack trace (first 10 frames only)
3. Take URL (without query params)
4. Concatenate: `message + stack + url`
5. Compute SHA256
6. Hex encode
7. Return as hash
```

**CRITICAL:** This is always the same. No timestamps. No random components. No variations.

**Code Pattern:**

```rust
pub fn compute_hash(error: &Error) -> String {
    let message = error.message.trim();
    let stack = error.stack
        .lines()
        .take(10)
        .collect::<Vec<_>>()
        .join("\n");
    let url = strip_query_params(&error.context.url);

    let input = format!("{}\n{}\n{}", message, stack, url);
    let digest = sha256(input.as_bytes());
    hex::encode(digest)
}

#[test]
fn test_hash_deterministic() {
    let error1 = Error::default();
    let error2 = Error::default();

    assert_eq!(compute_hash(&error1), compute_hash(&error2));
}

#[test]
fn test_hash_different_errors() {
    let error1 = Error { message: "Error A".into(), ..Default::default() };
    let error2 = Error { message: "Error B".into(), ..Default::default() };

    assert_ne!(compute_hash(&error1), compute_hash(&error2));
}
```

### 4.2 Ledger Immutability

**Rule:** Every INSERT to `ledger` table is permanent. No UPDATE, no DELETE, no mutations.

**Enforcement:**

```sql
CREATE TRIGGER ledger_immutable BEFORE UPDATE ON ledger
FOR EACH ROW EXECUTE FUNCTION raise_error('Ledger is immutable');

CREATE TRIGGER ledger_no_delete BEFORE DELETE ON ledger
FOR EACH ROW EXECUTE FUNCTION raise_error('Ledger cannot be deleted');
```

### 4.3 Error Deduplication

**Rule:** If error hash matches existing error in same project:

1. Do NOT insert new record
2. Increment `count` on existing record
3. Update `last_seen_at` to now
4. Append to ledger with event_type='duplicate'

**Code Pattern:**

```rust
pub async fn persist(project_id: &Uuid, error: NormalizedError) -> Result<StoredError> {
    let hash = compute_hash(&error);

    let existing = sqlx::query_as::<_, ErrorRecord>(
        "SELECT * FROM errors WHERE project_id = $1 AND hash = $2"
    )
    .bind(project_id)
    .bind(&hash)
    .fetch_optional(&pool)
    .await?;

    if let Some(existing) = existing {
        // Duplicate: increment count
        sqlx::query(
            "UPDATE errors SET count = count + 1, last_seen_at = NOW() WHERE id = $1"
        )
        .bind(existing.id)
        .execute(&pool)
        .await?;

        // Log to ledger
        sqlx::query(
            "INSERT INTO ledger (project_id, error_id, event_type, data) VALUES ($1, $2, $3, $4)"
        )
        .bind(project_id)
        .bind(existing.id)
        .bind("duplicate")
        .bind(json!({"count": existing.count + 1}))
        .execute(&pool)
        .await?;

        return Ok(StoredError {
            id: existing.id,
            was_duplicate: true,
            count: existing.count + 1,
        });
    }

    // New error: insert
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO errors (id, project_id, hash, message, stack, context, first_seen_at, last_seen_at, created_at) VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW(), NOW())"
    )
    .bind(id)
    .bind(project_id)
    .bind(&hash)
    .bind(&error.message)
    .bind(&error.stack)
    .bind(serde_json::to_value(&error.context)?)
    .execute(&pool)
    .await?;

    // Log to ledger
    sqlx::query(
        "INSERT INTO ledger (project_id, error_id, event_type) VALUES ($1, $2, $3)"
    )
    .bind(project_id)
    .bind(id)
    .bind("new_error")
    .execute(&pool)
    .await?;

    Ok(StoredError {
        id,
        was_duplicate: false,
        count: 1,
    })
}
```

### 4.4 Why Determinism Matters

### Problem: Sentry's Grouping

Sentry uses probabilistic heuristics. The same error sometimes groups with
different similar errors. This causes:

- Alert fatigue (same error appears in multiple groups)
- Noisy dashboards (false positive groupings)
- Manual merging (devs spend time cleaning up)

### Solution: Deterministic Hashing

FaultReport groups errors by cryptographic hash:

hash = SHA256(message + stack[:10] + url)

Same error always = same hash = same group.
Different errors always = different hash = different group.

This is:

- Provable (you can verify the hash)
- Auditable (inspect why errors grouped)
- Unchanging (same hash forever)
- Scalable (no ML models to retrain)

### Result

- Zero noise in error lists
- Zero false positive groupings
- Zero alert fatigue from duplicates
- Clean dashboards, immediately

### 4.5 ### IMMUTABLE HASH ALGORITHM (Constitutional)

The hash function MUST NEVER CHANGE once launched.

Permitted fields in hash:
✓ error.message (trimmed)
✓ error.stack[:10 frames only]
✓ error.context.url (no query params, no fragment)

FORBIDDEN in hash (no exceptions):
✗ timestamps
✗ IDs (error_id, user_id, session_id)
✗ browser version
✗ OS version
✗ environment
✗ context fields
✗ count
✗ first_seen_at
✗ any field that changes over time

If you need to group by environment/browser/OS:
→ Use separate indexing (not hash)
→ Use secondary grouping (not primary hash)
→ Never embed in the hash

BREAKING THIS RULE = PRODUCT DEATH

---

## 5. Data Flow (Golden Path)

### 5.1 Error Submission (Happy Path)

```
SDK submits error
    ↓
POST /api/projects/:projectId/errors
    ↓
Module A: Validate + Normalize
    ↓ (invalid) → HTTP 400
    ↓ (valid) → normalized error object
    ↓
Module B: Persist to DB
    ↓ (duplicate) → increment count
    ↓ (new) → insert new record
    ↓ (both cases) → append to ledger
    ↓
Check spike (count > 10 in 5 min)?
    ↓ (no) → return 201
    ↓ (yes) → trigger alert
    ↓
Module C: Post to Slack
    ↓ (success) → return 201
    ↓ (failure) → queue for retry, still return 201
    ↓
Return to SDK: { id, hash }
```

### 5.2 Dashboard View (Happy Path)

```
User loads dashboard.faultreport.io
    ↓
React frontend loads
    ↓
User logs in (Firebase)
    ↓
Frontend requests: GET /api/dashboard/projects/:projectId/errors
    ↓
Orchestrator checks authentication
    ↓ (invalid) → HTTP 401
    ↓ (valid) → continue
    ↓
Module D: Query errors from DB
    ↓
Return paginated error list
    ↓
Frontend renders table
```

---

## 6. Module APIs (Contracts)

### 6.1 Module A: ErrorCapture

**Input Type:**

```rust
pub struct RawError {
    pub message: String,
    pub stack: Option<String>,
    pub context: ErrorContext,
}

pub struct ErrorContext {
    pub url: String,
    pub browser: String,
    pub os: String,
    pub user_id: Option<String>,
}
```

**Output Type:**

```rust
pub struct NormalizedError {
    pub message: String,
    pub stack: String,
    pub context: ErrorContext,
    pub hash: String,
}
```

**Public Functions:**

```rust
pub fn validate(error: &RawError) -> Result<()>
pub fn normalize(error: &RawError) -> Result<NormalizedError>
pub fn compute_hash(error: &NormalizedError) -> String
```

### 6.2 Module B: Storage

**Public Functions:**

```rust
pub async fn persist(
    pool: &PgPool,
    project_id: Uuid,
    error: NormalizedError,
) -> Result<StoredError>

pub async fn get_error(
    pool: &PgPool,
    project_id: Uuid,
    error_id: Uuid,
) -> Result<ErrorRecord>

pub async fn list_errors(
    pool: &PgPool,
    project_id: Uuid,
    page: u32,
    per_page: u32,
) -> Result<Vec<ErrorRecord>>

pub async fn check_spike(
    pool: &PgPool,
    project_id: Uuid,
    hash: &str,
) -> Result<bool>
```

### 6.3 Module C: Alert

**Public Functions:**

```rust
pub async fn trigger(
    project_id: Uuid,
    error_id: Uuid,
    rule_id: Uuid,
) -> Result<AlertResponse>
```

### 6.4 Module D: Dashboard

**Public Endpoints:**

```
GET /api/dashboard/projects/:projectId/errors
GET /api/dashboard/projects/:projectId/errors/:errorId
POST /auth/login
POST /auth/logout
```

---

## 7. Non-Negotiable Rules

### 7.1 Code Style

- **Rust:** rustfmt default format, clippy no warnings
- **SQL:** UPPERCASE keywords, snake_case identifiers
- **React:** Functional components only, hooks only, no class components
- **TypeScript:** `strict: true` in tsconfig

### 7.2 Errors

- **All errors must implement Display + Debug**
- **All async functions must return Result<T, Error>**
- **All errors propagate to caller (use ? operator)**
- **Never unwrap() in production code**

### 7.3 Logging

- **Use tracing crate for all logs**
- **Log all errors at ERROR level**
- **Log all state changes at INFO level**
- **Log performance metrics at DEBUG level**
- **Never log sensitive data (passwords, tokens, API keys)**

### 7.4 Testing

- **100% of public functions must have tests**
- **All tests are deterministic (no randomness)**
- **All tests use fixtures (no external dependencies)**
- **Test names describe the assertion: `test_<function>_<input>_<expected>`**

### 7.5 Database

- **All schema changes require migration files**
- **Migrations are version-numbered: `001_initial_schema.sql`**
- **Every table has `created_at` timestamp**
- **No cascading deletes (use soft deletes instead)**
- **All queries use parameterized statements (never string interpolation)**

### 7.6 API

- **All endpoints return JSON**
- **All errors return JSON: `{ "error": "message" }`**
- **All paginated responses include: `total`, `page`, `per_page`**
- **All timestamps are ISO 8601 UTC**
- **All IDs are UUIDs (never sequential integers)**

### 7.7 Secrets

- **Never commit `.env` files**
- **All secrets via environment variables**
- **Never log secrets**
- **Never pass secrets in URLs**
- **Rotate secrets quarterly**

---

## 8. Testing Requirements

### 8.1 Unit Tests

**Location:** Same file as code, `#[cfg(test)]` module

**Example:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash_deterministic() {
        // Test determinism
    }

    #[test]
    fn test_validate_rejects_empty_message() {
        // Test validation
    }

    #[tokio::test]
    async fn test_persist_new_error() {
        // Test async behavior
    }
}
```

### 8.2 Integration Tests

**Location:** `tests/` directory

**Example:**

```rust
#[tokio::test]
async fn test_error_submission_flow() {
    // Setup
    let pool = setup_test_db().await;
    let client = setup_test_client().await;

    // Act
    let response = client
        .post("/api/projects/test-project/errors")
        .json(&test_error())
        .send()
        .await;

    // Assert
    assert_eq!(response.status(), 201);
    assert!(response.json::<Value>()["id"].is_string());
}
```

### 8.3 Test Data

**Use fixtures, not randomness:**

```rust
fn test_error() -> RawError {
    RawError {
        message: "Cannot read property 'x' of undefined".into(),
        stack: Some("at foo (app.js:42)".into()),
        context: ErrorContext {
            url: "https://example.com/page".into(),
            browser: "Chrome 120".into(),
            os: "macOS".into(),
            user_id: Some("user123".into()),
        },
    }
}
```

---

## 9. Deployment Rules

### 9.1 Docker

**Every module must have a Dockerfile:**

```dockerfile
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/faultreport /app/
CMD ["/app/faultreport"]
```

**Rules:**

- Multi-stage build (builder + runtime)
- No `latest` tags (use semantic versions)
- Health check endpoint at `/health`

### 9.2 Environment Variables

**Required for all deployments:**

```
DATABASE_URL=postgresql://...
STRIPE_SECRET_KEY=sk_...
FIREBASE_PROJECT_ID=...
SLACK_WEBHOOK_URL=...
RUST_LOG=info
```

**No hardcoded values. Ever.**

### 9.3 Deployment Steps

1. **Build:** `docker build -t faultreport:1.0.0 .`
2. **Test:** Run tests in container
3. **Push:** `docker push registry/faultreport:1.0.0`
4. **Deploy:** Railway automatically deploys on push
5. **Verify:** Check health endpoint: `curl /health`

### 9.4 Rollback Procedure

1. **Identify previous version:** Check Docker tag history
2. **Redeploy:** `docker run registry/faultreport:previous-version`
3. **Verify:** Check health endpoint
4. **Investigate:** Look at logs for what went wrong

---

## 10. Recovery Procedure

### 10.1 If Database Corrupts

1. **Stop application**
2. **Create backup:** `pg_dump > backup.sql`
3. **Restore from last known good backup**
4. **Replay ledger:** Replay all ledger entries since backup
5. **Verify integrity:** Run deterministic replay tests
6. **Restart application**

### 10.2 If Application Crashes

1. **Check logs:** `docker logs faultreport`
2. **If recoverable:** Just restart (`docker restart faultreport`)
3. **If not:** Redeploy last known good version
4. **If persistent:** Investigate root cause before redeploying

### 10.3 If Authentication Fails

1. **Verify Firebase config** (check FIREBASE_PROJECT_ID)
2. **Verify service account** (download new one from Firebase console)
3. **Restart application**

### 10.4 If Stripe Integration Breaks

1. **Check STRIPE_SECRET_KEY is set**
2. **Verify webhook URL in Stripe console**
3. **Retry failed payments manually via Stripe dashboard**

---

## Summary

**This document is the law.**

Every line of code must obey these rules. Every deployment must follow these procedures. Every module must respect these boundaries.

If you're uncertain about anything not explicitly stated here, **ask first before coding.**

The goal: A burned building, one document, one engineer, full rebuild in one week.

---

**Document Version:** 1.0  
**Last Updated:** 2024-01-15  
**Authority:** Founder  
**Status:** IMMUTABLE
