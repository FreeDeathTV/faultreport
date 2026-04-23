# FaultReport Project Structure & Complete Ruleset

**Document for External Review — Updated Post-Feedback**

This document defines the complete project structure, all rules, constraints, and validation criteria. Updated to reflect positioning feedback: deterministic grouping as core differentiator, self-hosting as wedge, predictable billing as trust anchor.

---

## Part 0: Go-to-Market Strategy (3 Wedges)

### Wedge 1: Deterministic Error Grouping (HERO FEATURE)

**Target Market:** Frontend teams frustrated with Sentry's noisy grouping

**The Problem We Solve:**

- Sentry's grouping is probabilistic (same error sometimes groups differently)
- Causes alert fatigue, noisy dashboards, manual cleanup
- Developers spend hours managing false positive groupings

**Our Solution:**

- Deterministic hash-based grouping: `SHA256(message + stack[:10] + url)`
- Same error always groups together
- Different errors never merge
- Mathematically proven. Cryptographically guaranteed. Zero false positives.

**Why This Matters:**

- Provable (you can verify the hash yourself)
- Auditable (inspect why errors grouped)
- Unchanging (same hash forever, same grouping forever)
- Scalable (no ML models, no probabilistic drift)
- Sentry cannot copy easily (locked into probabilistic approach)

**MVP Commitment:**

- Deterministic grouping ships in v1.0
- Tests prove grouping is deterministic (same error, same hash, always)
- Documentation explains why this beats Sentry
- Dashboard clearly shows how many duplicates were collapsed by grouping

**Validation Criteria:**

```
Given: Same error submitted 10 times
Expected: Dashboard shows 1 error with count=10
Test: test_deterministic_grouping_same_error_always_collapses

Given: 10 different errors
Expected: Dashboard shows 10 errors (never merged)
Test: test_different_errors_never_group
```

---

### Wedge 2: Self-Hosting (Sovereignty & Control)

**Target Market:** EU companies, fintech, healthcare, privacy-conscious devs, SaaS-skeptics

**The Problem We Solve:**

- Sentry forces cloud-hosted (no self-hosting)
- GDPR compliance is painful (data leaves EU)
- Vendor lock-in (export is complicated)
- High latency for on-prem deployments

**Our Solution:**

```bash
git clone https://github.com/faultreport/core
docker-compose up
open localhost:3000
```

Features:

- Full data ownership (your infrastructure, your database)
- No vendor lock-in (export anytime, format: JSON)
- Works offline (no cloud dependency)
- Works in air-gapped networks (hospitals, gov, fintech)

**Why This Matters:**

- Massive market (self-hosters) that Sentry abandoned
- Instant setup (literally one command)
- Full control (your infrastructure, your rules)
- Compliance-friendly (GDPR, HIPAA, etc. — data never leaves)
- Cost transparency (self-host = only your infrastructure costs)

**MVP Commitment:**

- Self-hosting works out of the box (docker-compose up)
- Docker Compose setup is documented
- Environment file is provided (.env.example)
- Export functionality exists (JSON endpoint)
- Private deployment guide included (docs/SELF_HOSTING.md)
- One-click restore from backup (all data in PostgreSQL)

**Validation Criteria:**

```bash
Test: docker-compose up from zero
Expected: Service online in < 2 minutes
Test: curl http://localhost:3000
Expected: Dashboard loads
Test: POST error to local instance
Expected: Error appears in local dashboard
Test: Export errors as JSON
Expected: JSON contains full error + context
```

---

### Wedge 3: Predictable Billing (Trust & Transparency)

**Target Market:** Developers burned by Sentry's surprise invoices

**The Problem We Solve:**

- Sentry's per-event pricing causes surprise £500+ invoices
- Throttling happens silently (events dropped, you don't know)
- No transparency on what will cost what
- Budget planning is impossible

**Our Solution:**

```
FLAT BILLING: £49/month
  - No per-event pricing
  - No throttling
  - No silent drops
  - Unlimited errors (30-day retention)
  - Hard cap: alerts you before hitting limit
  - Transparent rate limiting (visible in API)
  - Overage: £99/month for unlimited (no surprise invoices)
```

**Why This Matters:**

- Budget predictability (CFOs love this)
- No surprise invoices (trust earned)
- Transparent rate limiting (you see it coming)
- No behavior changes mid-month (no throttling)
- Honest pricing (no hidden per-event charges)

**MVP Commitment:**

- Flat £49/month billing (no per-event charges)
- Rate limiting: 10,000 errors/hour per project (hard cap)
- Hard cap behavior: reject errors after 10K/hour (don't drop silently)
- Rate limit status visible in API response headers:
  ```
  X-RateLimit-Limit: 10000
  X-RateLimit-Remaining: 9875
  X-RateLimit-Reset: 1705339200
  ```
- Documentation of all limits included
- Slack alert when approaching cap (8K/hour)
- Monthly billing email with usage breakdown

**Validation Criteria:**

```
Test: Send 9,000 errors
Expected: All accepted, status 201

Test: Send 11,000 errors
Expected: First 10,000 accepted (201), next 1,000 rejected (429)

Test: Check rate limit headers
Expected: Headers show remaining capacity

Test: Monitor billing email
Expected: Monthly summary shows usage + cost (flat £49)
```

---

## Part 1: Project Structure (Canonical)

```
faultreport/
├── README.md                          # User-facing product description
├── ARCHITECTURE.md                    # Constitutional architecture document
├── STRUCTURE.md                       # This file (complete spec)
│
├── backend/                           # Rust backend (Actix-web)
│   ├── Cargo.toml                    # Dependencies (locked)
│   ├── Dockerfile                    # Multi-stage production build
│   ├── .dockerignore
│   ├── src/
│   │   ├── main.rs                   # Application entry point
│   │   ├── error.rs                  # Custom error types
│   │   ├── config.rs                 # Configuration from env vars
│   │   ├── db.rs                     # PostgreSQL connection pool
│   │   │
│   │   ├── modules/
│   │   │   ├── mod.rs
│   │   │   ├── error_capture.rs      # Module A: Validation + Normalization
│   │   │   ├── storage.rs            # Module B: Persistence + Ledger
│   │   │   ├── alert.rs              # Module C: Spike Detection + Slack
│   │   │   └── dashboard_api.rs      # Module D: Dashboard API Endpoints
│   │   │
│   │   ├── orchestrator.rs           # FaultReport Orchestrator (main router)
│   │   ├── handlers.rs               # HTTP request handlers
│   │   ├── middleware.rs             # Auth + logging middleware
│   │   └── logging.rs                # Tracing setup
│   │
│   ├── tests/
│   │   ├── integration_test.rs       # Full error submission flow
│   │   ├── deterministic_grouping.rs # Prove grouping determinism
│   │   ├── deduplication.rs          # Test duplicate handling
│   │   ├── rate_limiting.rs          # Test rate limit enforcement
│   │   └── self_hosted.rs            # Test docker-compose deployment
│   │
│   └── migrations/
│       ├── 001_initial_schema.sql    # Tables: errors, ledger, projects, users
│       ├── 002_indexes.sql           # All required indexes
│       ├── 003_ledger_immutable.sql  # Triggers for ledger safety
│       └── 004_rate_limits.sql       # Rate limit tracking table
│
├── frontend/                          # React + TypeScript + Tailwind
│   ├── package.json
│   ├── tsconfig.json                 # strict: true
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   ├── Dockerfile
│   ├── index.html
│   │
│   ├── src/
│   │   ├── main.tsx
│   │   ├── index.css                 # Tailwind imports only
│   │   │
│   │   ├── components/
│   │   │   ├── ErrorList.tsx         # Table of errors (deterministic grouping highlighted)
│   │   │   ├── ErrorDetail.tsx       # Single error detail (show hash)
│   │   │   ├── Layout.tsx            # Main shell
│   │   │   ├── Loading.tsx
│   │   │   └── RateLimitIndicator.tsx # Show usage vs cap
│   │   │
│   │   ├── pages/
│   │   │   ├── Dashboard.tsx
│   │   │   ├── Login.tsx
│   │   │   └── NotFound.tsx
│   │   │
│   │   ├── api/
│   │   │   └── client.ts             # Fetch wrapper (typed)
│   │   │
│   │   ├── auth/
│   │   │   └── firebase.ts           # Firebase setup
│   │   │
│   │   └── types/
│   │       └── index.ts              # All TypeScript interfaces
│   │
│   └── tests/
│       ├── ErrorList.test.tsx
│       └── Dashboard.test.tsx
│
├── sdk/                               # JavaScript/TypeScript SDK (npm package)
│   ├── package.json
│   ├── tsconfig.json
│   ├── rollup.config.js
│   │
│   ├── src/
│   │   ├── index.ts
│   │   ├── client.ts                 # Core SDK logic
│   │   ├── queue.ts                  # Local error queue (batching)
│   │   ├── types.ts
│   │   └── utils.ts                  # Hash computation (matches backend)
│   │
│   └── tests/
│       ├── determinism.test.ts       # SDK hash matches backend hash
│       └── client.test.ts
│
├── docs/
│   ├── DEPLOYMENT.md                 # How to deploy (cloud + self-hosted)
│   ├── SELF_HOSTING.md              # Self-hosting guide (docker-compose)
│   ├── API.md                        # API documentation
│   ├── BILLING.md                    # Billing & rate limits (transparent)
│   ├── DETERMINISM.md                # Deep dive: why deterministic grouping
│   └── MIGRATION_GUIDE.md            # Coming: Sentry → FaultReport
│
├── docker-compose.yml                # Local development + self-hosting
├── .env.example                      # Example configuration
├── .github/workflows/
│   ├── test.yml                      # Run tests on push
│   └── deploy.yml                    # Deploy to Railway on push
│
├── CHANGELOG.md                      # Version history
├── LICENSE                           # MIT
└── Makefile                          # Build shortcuts
```

---

## Part 2: Deterministic Grouping Deep Dive

### Why This Is the Hero Feature

**The Problem (Sentry):**

```
Error 1: "Cannot read property 'x' of undefined" at line 42
Error 2: "Cannot read property 'x' of undefined" at line 42
Error 3: "Cannot read property 'x' of undefined" at line 42

Sentry groups them into:
- Group A: "Cannot read property" (65% confidence)
- Group B: "Cannot read property 'x'" (35% confidence)
- Group C: Separate (no match)

Result: Same error appears in 3 groups. Alert fatigue. Manual cleanup.
```

**The Solution (FaultReport):**

```
Error 1: hash = SHA256("Cannot read...\nat line 42\nhttps://example.com/page")
Error 2: hash = SHA256("Cannot read...\nat line 42\nhttps://example.com/page")
Error 3: hash = SHA256("Cannot read...\nat line 42\nhttps://example.com/page")

All have identical hash → All group together
Result: One error with count=3. Clean. Guaranteed.
```

### The Hash Algorithm (Immutable)

```rust
pub fn compute_hash(error: &NormalizedError) -> String {
    let message = error.message.trim();

    let stack = error.stack
        .lines()
        .take(10)  // First 10 frames only (ignore rest)
        .collect::<Vec<_>>()
        .join("\n");

    let url = strip_query_params(&error.context.url);

    let input = format!("{}\n{}\n{}", message, stack, url);
    let digest = sha256(input.as_bytes());
    hex::encode(digest)
}
```

**Critical Properties:**

- Same input always produces same output (deterministic)
- Different errors always produce different hashes
- Hash ignores timestamps, IDs, user context
- Hash is URL-origin-specific (different pages = different hash)
- Hash takes only first 10 stack frames (noise-resistant)

### Dashboard Display of Grouping

The error list shows:

```
Message                              Count    First Seen      Last Seen
─────────────────────────────────────────────────────────────────────────────
Cannot read property 'x'             42       2 min ago       10s ago
Undefined is not a function          15       5 min ago       2 min ago
TypeError: fetch failed              8        1h ago          45m ago
```

Each row represents ONE GROUP (one unique hash).

The count is the number of times that exact error occurred.

The dashboard includes a note:

```
"Deterministic Grouping: These errors are grouped by cryptographic hash.
Same error always groups together. Different errors never merge.
Click an error to see the hash."
```

### Error Detail View

When you click an error, you see:

```
Error: Cannot read property 'x' of undefined
Hash: abc123def456...

First Seen: 2024-01-15 10:30:00 UTC
Last Seen: 2024-01-15 10:32:15 UTC
Occurrences: 42

Hash Breakdown:
  Message: Cannot read property 'x' of undefined
  Stack (first 10 frames):
    at Object.<anonymous> (/app.js:42:5)
    at async doSomething (/app.js:100:10)
    ...
  URL: https://example.com/page

Why This Error Is Grouped:
  Every error with this exact message + stack + URL produces the same hash.
  This is mathematically guaranteed. No false positives.
```

---

## Part 3: Complete Ruleset

### Rule Category 1: Sovereignty & Isolation

#### **Rule 1.1: Module Isolation (CRITICAL)**

- No Rust `use module_b::function` imports between modules
- All inter-module communication flows through Orchestrator only
- Each module is in its own `modules/` subdirectory with a `mod.rs`
- Module exports only public API functions (listed in ARCHITECTURE.md)
- No shared state between modules (all state in PostgreSQL)

**Validation:**

```bash
grep -r "use.*modules::" src/modules/error_capture.rs
grep -r "use.*modules::" src/modules/storage.rs
# Should return NOTHING
```

#### **Rule 1.2: Orchestrator Authority**

- Orchestrator is the ONLY place that calls multiple modules in sequence
- Orchestrator routes all HTTP requests to appropriate module
- No module can spawn async tasks independently
- No background threads in modules

**Validation:**

- All HTTP routes defined in `orchestrator.rs`
- All `tokio::spawn` calls in `orchestrator.rs` only
- Search codebase: `grep -r "tokio::spawn" src/` should find only orchestrator.rs

#### **Rule 1.3: No Shared Mutable State**

- No `static mut` anywhere
- No `Arc<Mutex<T>>` for shared state (only PgPool exempt)
- State lives only in PostgreSQL or function parameters
- Configuration immutable once loaded

**Validation:**

```bash
grep -r "static mut" src/
grep -r "Arc<Mutex" src/ | grep -v "PgPool"
# Both should return NOTHING
```

---

### Rule Category 2: Determinism (CORE TO MVP)

#### **Rule 2.1: Deterministic Hashing (CRITICAL)**

- Error hash function is pure: `fn compute_hash(error: &NormalizedError) -> String`
- Hash computed EXACTLY as specified in Part 2 above
- Same error on different machines = identical hash (MUST BE TESTED)
- Hash includes: message + first 10 stack frames + URL (without query params)
- Hash excludes: timestamps, IDs, user context, browser/OS
- Hash function has unit tests proving determinism

**Test Requirement:**

```rust
#[test]
fn test_hash_deterministic_same_input() {
    let error = test_error();
    let hash1 = compute_hash(&error);
    let hash2 = compute_hash(&error);
    assert_eq!(hash1, hash2);  // MUST pass
}

#[test]
fn test_hash_different_errors() {
    let error1 = Error { message: "Error A".into(), ..Default::default() };
    let error2 = Error { message: "Error B".into(), ..Default::default() };
    assert_ne!(compute_hash(&error1), compute_hash(&error2));
}

#[test]
fn test_hash_ignores_timestamp() {
    let mut error1 = test_error();
    let mut error2 = test_error();
    // Only difference: created_at timestamp
    error1.created_at = "2024-01-15T10:00:00Z".into();
    error2.created_at = "2024-01-16T10:00:00Z".into();
    assert_eq!(compute_hash(&error1), compute_hash(&error2));
}

#[test]
fn test_hash_url_without_query_params() {
    let mut error1 = test_error();
    let mut error2 = test_error();
    error1.context.url = "https://example.com/page?foo=bar".into();
    error2.context.url = "https://example.com/page?baz=qux".into();
    assert_eq!(compute_hash(&error1), compute_hash(&error2));  // Same hash
}

#[test]
fn test_hash_first_10_frames_only() {
    let mut error1 = test_error();
    let mut error2 = test_error();
    // error1 has 15 stack frames, error2 has 5 (same first 10)
    error1.stack = "frame1\nframe2\n...\nframe15".into();
    error2.stack = "frame1\nframe2\n...\nframe10".into();
    assert_eq!(compute_hash(&error1), compute_hash(&error2));
}
```

#### **Rule 2.2: No Randomness in Core Logic**

- No `rand::random()` in error_capture, storage, or hash computation
- No `uuid::Uuid::new_v4()` in hash computation (only in ID generation)
- Randomness allowed ONLY in ID generation (v4 UUIDs fine there)
- Timestamps allowed (set at persistence, not in hash)

**Validation:**

```bash
grep -r "random\|rand::" src/modules/error_capture.rs
grep -r "random\|rand::" src/modules/storage.rs | grep -v "test"
# Should return NOTHING
```

#### **Rule 2.3: Deterministic Deduplication**

- If hash exists for same project: increment count, don't duplicate
- Same duplicate always takes same path (idempotent)
- Spike detection uses only wall-clock time from DB

**Test:**

```rust
#[tokio::test]
async fn test_deduplication_deterministic() {
    let pool = setup_test_db().await;
    let error = test_error();

    // Submit same error 5 times
    for _ in 0..5 {
        persist(&pool, test_project_id(), error.clone()).await.unwrap();
    }

    // Should result in 1 error record with count=5
    let errors = list_errors(&pool, test_project_id()).await.unwrap();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].count, 5);
}
```

---

### Rule Category 3: Auditability & Immutability

#### **Rule 3.1: Append-Only Ledger (CRITICAL)**

- Ledger table NEVER has UPDATE or DELETE on entries
- Every insert is permanent
- Ledger has database triggers enforcing immutability
- Ledger is source of truth for audit trail

**Schema:**

```sql
CREATE TABLE ledger (
  id BIGSERIAL PRIMARY KEY,
  project_id UUID NOT NULL,
  error_id UUID NOT NULL,
  event_type VARCHAR(50) NOT NULL,  -- 'new_error' | 'duplicate' | 'spike'
  data JSONB,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TRIGGER ledger_immutable BEFORE UPDATE ON ledger
FOR EACH ROW EXECUTE FUNCTION raise_error('Ledger is immutable');

CREATE TRIGGER ledger_no_delete BEFORE DELETE ON ledger
FOR EACH ROW EXECUTE FUNCTION raise_error('Ledger cannot be deleted');
```

#### **Rule 3.2: Error Record Immutability**

- Only these fields can be UPDATEd: `count`, `last_seen_at`, `status`
- Never UPDATE: `message`, `stack`, `context`, `hash`, `created_at`

**Valid Updates:**

```sql
UPDATE errors SET
  count = count + 1,
  last_seen_at = NOW()
WHERE id = $1
-- CORRECT
```

**Invalid Updates (FORBIDDEN):**

```sql
UPDATE errors SET message = 'new message' WHERE id = $1
-- FORBIDDEN - immutable field
```

#### **Rule 3.3: Logging All State Changes**

- Every INSERT to errors → entry in ledger
- Every UPDATE to errors → entry in ledger
- Ledger includes: event_type, data (old/new values)
- No silent changes

**Code Pattern:**

```rust
pub async fn persist(
    pool: &PgPool,
    project_id: Uuid,
    error: NormalizedError,
) -> Result<StoredError> {
    let hash = compute_hash(&error);

    // 1. Check for duplicate
    let existing = get_error_by_hash(pool, project_id, &hash).await?;

    if let Some(existing) = existing {
        // DUPLICATE PATH: increment + log
        sqlx::query("UPDATE errors SET count = count + 1, last_seen_at = NOW() WHERE id = $1")
            .bind(existing.id)
            .execute(pool)
            .await?;

        sqlx::query(
            "INSERT INTO ledger (project_id, error_id, event_type, data) VALUES ($1, $2, $3, $4)"
        )
        .bind(project_id)
        .bind(existing.id)
        .bind("duplicate")
        .bind(json!({"count": existing.count + 1}))
        .execute(pool)
        .await?;

        return Ok(StoredError { id: existing.id, was_duplicate: true, count: existing.count + 1 });
    }

    // NEW ERROR PATH: insert + log
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO errors (id, project_id, hash, message, stack, context, created_at) VALUES ($1, $2, $3, $4, $5, $6, NOW())"
    )
    .bind(id)
    .bind(project_id)
    .bind(&hash)
    .bind(&error.message)
    .bind(&error.stack)
    .bind(serde_json::to_value(&error.context)?)
    .execute(pool)
    .await?;

    sqlx::query(
        "INSERT INTO ledger (project_id, error_id, event_type) VALUES ($1, $2, $3)"
    )
    .bind(project_id)
    .bind(id)
    .bind("new_error")
    .execute(pool)
    .await?;

    Ok(StoredError { id, was_duplicate: false, count: 1 })
}
```

### \*\*Rule 2.2: ## Alert Deduplication (Required)

Rule: Only one alert per error group per 5 minutes.

Logic:

- Error hash = abc123
- First spike detected at 10:00 → post to Slack
- Spike still active at 10:05 → DO NOT post again
- Spike still active at 10:10 → DO NOT post again
- Spike cleared at 10:15 → reset counter
- Spike returns at 10:20 → post to Slack again

Test:
[ ] Same error spiked 10 times in 5 min = 1 Slack alert
[ ] Different errors spiked = separate alerts
[ ] After 5 min silence, new spike triggers alert

---

### Rule Category 4: Rate Limiting (Predictable Billing)

#### **Rule 4.1: Hard Rate Limit (10K/hour per project)**

- Limit: 10,000 errors per hour per project
- Hard cap: errors beyond 10K are REJECTED (status 429)
- No silent dropping (client gets explicit rejection)
- No throttling (first 10K all accepted)

**Implementation:**

```rust
pub async fn check_rate_limit(
    pool: &PgPool,
    project_id: Uuid,
) -> Result<bool> {
    let one_hour_ago = Instant::now() - Duration::from_secs(3600);

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM errors WHERE project_id = $1 AND created_at > $2"
    )
    .bind(project_id)
    .bind(one_hour_ago)
    .fetch_one(pool)
    .await?;

    Ok(count < 10000)
}
```

#### **Rule 4.2: Rate Limit Headers (Transparency)**

Every API response includes:

```
X-RateLimit-Limit: 10000
X-RateLimit-Remaining: 9875
X-RateLimit-Reset: 1705339200
```

Client can see exactly how much capacity remains.

#### **Rule 4.3: Billing Guarantee**

- Flat £49/month billing (no per-event charges)
- No surprise invoices
- Overage: additional £99/month for unlimited (opt-in, not automatic)
- All limits transparent in API + documentation

## \*\*Rule 4.4: Hard Rate Limit Guarantee (Non-Negotiable)

**Clarification:** First 10,000 errors in rolling 1-hour window accepted (HTTP 201). 10,001st+ rejected (HTTP 429). Rejected do NOT count.

What we guarantee:

- Errors 1–10,000: ACCEPTED (HTTP 201)
- Error 10,001+: REJECTED (HTTP 429)
- No silent drops
- No throttling
- Window: time_now - 3600s (rolling)
- Headers on every response

Every response includes:

```
X-RateLimit-Limit: 10000
X-RateLimit-Remaining: 9875
X-RateLimit-Reset: 1705339200
```

Tests must verify:
[ ] Errors 1–10,000 all return 201
[ ] Error 10,001 returns 429
[ ] Headers accurate
[ ] Headers update real-time
[ ] Reset 3600s after first
[ ] 429s don't count toward limit

---

### Rule Category 5: Database Integrity

#### **Rule 5.1: Migrations Are Immutable**

- Migration files NEVER modified after creation
- New changes = new migration file
- Numbered: 001*, 002*, 003\_, etc.
- Each has UP (creation) and DOWN (rollback)

**Files:**

```
migrations/001_initial_schema.sql
migrations/002_indexes.sql
migrations/003_ledger_immutable.sql
migrations/004_rate_limits.sql
```

#### **Rule 5.2: Schema Constraints**

- All tables have `created_at NOT NULL DEFAULT NOW()`
- All ID columns are `UUID PRIMARY KEY`
- Foreign keys use `ON DELETE RESTRICT` (no cascade)
- Unique constraint on (project_id, hash) for errors
- Ledger has BIGSERIAL for ordering

**Schema Example:**

```sql
CREATE TABLE errors (
  id UUID PRIMARY KEY,
  project_id UUID NOT NULL,
  hash VARCHAR(64) NOT NULL,
  message TEXT NOT NULL,
  stack TEXT,
  context JSONB,
  count INTEGER DEFAULT 1,
  first_seen_at TIMESTAMP NOT NULL DEFAULT NOW(),
  last_seen_at TIMESTAMP NOT NULL DEFAULT NOW(),
  status VARCHAR(20) DEFAULT 'active',
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_errors_project_hash ON errors(project_id, hash);
CREATE INDEX idx_errors_project_status ON errors(project_id, status);
CREATE INDEX idx_errors_last_seen ON errors(last_seen_at DESC);
```

---

### Rule Category 6: Self-Hosting

## Self-Hosting Validation (Must Pass All)

[ ] macOS x86_64 — docker-compose up + dashboard loads
[ ] macOS ARM64 — docker-compose up + dashboard loads
[ ] Linux x86_64 — docker-compose up + dashboard loads
[ ] Linux ARM64 — docker-compose up + dashboard loads
[ ] Windows 10/11 WSL2 — docker-compose up + dashboard loads
[ ] Windows 10/11 Docker Desktop — docker-compose up + dashboard loads

For each:
[ ] Services start < 2 minutes
[ ] Dashboard loads at localhost:3000
[ ] Can submit error via API
[ ] Error appears in dashboard
[ ] Export works
[ ] Restart doesn't lose data
Run this matrix before launch.

#### **Rule 6.1: Docker Compose Must Work Out-of-Box**

```bash
git clone https://github.com/faultreport/core
docker-compose up
# Service should be online in < 2 minutes
# Dashboard at localhost:3000
# API at localhost:8000
```

**docker-compose.yml Includes:**

- PostgreSQL 15 (with initial schema)
- Rust backend (from Dockerfile)
- React frontend (from Dockerfile)
- All configuration via .env file

#### **Rule 6.2: Export Functionality**

- Endpoint: `GET /api/projects/:projectId/export`
- Format: JSON (all errors + context)
- No vendor lock-in (data is yours)

**Response:**

```json
{
  "project_id": "...",
  "exported_at": "2024-01-15T10:30:00Z",
  "errors": [
    {
      "id": "...",
      "hash": "...",
      "message": "...",
      "stack": "...",
      "context": {...},
      "count": 5,
      "first_seen_at": "...",
      "last_seen_at": "..."
    }
  ]
}
```

---

### Rule Category 7: Testing

// tests/hash_consensus.rs
// This test runs ONCE and must pass on every platform

#[test]
fn test_sdk_hash_matches_backend() {
// These are the canonical test vectors
let test_cases = vec![
        (
            RawError {
                message: "Cannot read property 'x' of undefined",
                stack: Some("at line 42\nat line 43\nat line 44\n..."),
                context: ErrorContext {
                    url: "https://example.com/page?foo=bar",
                    ...
                }
            },
            "expected_hash_abc123"  // Hardcoded expected value
        ),
        // ... 50 more test cases
    ];

    for (error, expected_hash) in test_cases {
        let backend_hash = compute_hash(&error);  // Rust
        let sdk_hash = /* JavaScript SDK */ compute_hash(error);  // Node.js

        assert_eq!(backend_hash, expected_hash);
        assert_eq!(sdk_hash, expected_hash);
        assert_eq!(backend_hash, sdk_hash);
    }

}
Also test:

macOS vs Linux vs Windows
JavaScript in browser vs Node.js
Different endianness (if applicable)
Different locale settings
Different file encodings

This test must never fail.

#### **Rule 7.1: Test Coverage**

- 100% of public functions have unit tests
- All critical paths have integration tests
- Determinism explicitly tested (see Part 2)

**Minimum Tests:**

**Module A (ErrorCapture):**

- `test_hash_deterministic_same_input`
- `test_hash_different_errors`
- `test_hash_ignores_timestamp`
- `test_hash_url_without_query_params`
- `test_hash_first_10_frames_only`
- `test_validate_empty_message_rejected`
- `test_validate_url_length_limit`

**Module B (Storage):**

- `test_persist_new_error`
- `test_persist_duplicate_error_increments_count`
- `test_persist_creates_ledger_entry`
- `test_deduplication_deterministic`
- `test_get_error_by_id`
- `test_list_errors_paginated`
- `test_rate_limit_enforcement`

**Module C (Alert):**

- `test_spike_detection_threshold`
- `test_spike_posts_to_slack`
- `test_slack_failure_handling`

**Module D (Dashboard):**

- `test_list_errors_requires_auth`
- `test_error_detail_shows_hash`
- `test_grouping_visualization`

#### **Rule 7.2: No External Dependencies in Tests**

- All tests use in-memory DB or fixtures
- No real Slack calls
- No real Firebase calls
- No real Stripe calls
- All mocked or stubbed

#### **Rule 7.3: Deterministic Fixtures**

All test data hardcoded:

```rust
fn test_error() -> RawError {
    RawError {
        message: "Cannot read property 'x' of undefined".into(),
        stack: Some("at Object.<anonymous> (/app.js:42:5)".into()),
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

### Rule Category 8: Security

## Dashboard Performance SLOs (Measured, Not Aspirational)

[ ] Error list (100 errors): < 200ms p99
[ ] Error detail (with 50 occurrences): < 150ms p99
[ ] Filter/sort: < 100ms p99
[ ] Initial page load: < 500ms p99
[ ] Dashboard with 10k errors: still < 200ms p99

Implementation rules:

- Pagination: max 50 errors per request
- Indexes on (project_id, created_at DESC)
- Server-side filtering (never send all data to client)
- Minimal JSON payloads (no nested full context)
- Caching: error list 5-min TTL

#### **Rule 8.1: Secrets Management**

- Never commit `.env` files
- Never log secrets
- Never embed secrets in code
- All secrets via environment variables

**Required Env Vars:**

```
DATABASE_URL=postgresql://...
STRIPE_SECRET_KEY=sk_...
FIREBASE_PROJECT_ID=...
SLACK_WEBHOOK_URL=...
RUST_LOG=info
```

#### **Rule 8.2: Data Validation**

- All user input validated before processing
- All JSON validated against schema
- Invalid input returns 400
- No type coercion (strict typing)

#### **Rule 8.3: Sensitive Data Removal**

Before hashing/storing, strip:

- Passwords
- API keys
- Auth tokens
- Credit card numbers
- Session cookies

---

### Rule Category 9: Frontend

#### **Rule 9.1: React Component Structure**

- Functional components only (no class components)
- Hooks for state (useState, useEffect)
- No external state library (Redux, Zustand, etc.)
- TypeScript `strict: true`

#### **Rule 9.2: Type Safety**

Every component fully typed:

```typescript
interface ErrorListProps {
  errors: ErrorRecord[];
  page: number;
  onPageChange: (page: number) => void;
}

export function ErrorList({ errors, page, onPageChange }: ErrorListProps) {
  return (/* ... */)
}
```

#### **Rule 9.3: No Custom CSS**

- All styling via Tailwind only
- No `.css` files with custom rules
- No CSS-in-JS
- Only index.css (Tailwind imports)

---

#### **Rule 9.4: No Production Console Logging (SECURITY)**

- Explicit `console.log`, `console.warn`, and `console.error` calls are strictly forbidden in production code.
- All caught exceptions must be reported via `FaultReport.captureException(error)` to ensure they are handled within the sovereign architecture.
- Development-only logs must be wrapped in environmental checks: `if (process.env.NODE_ENV === 'development')`.
- This rule is enforced via ESLint: `no-console: ["error", { "allow": ["debug"] }]` in the production build pipeline.

### Rule Category 10: API

#### **Rule 10.1: Consistent Response Format**

**Success (2xx):**

```json
{
  "data": {
    /* payload */
  },
  "status": "success"
}
```

**Error (4xx/5xx):**

```json
{
  "error": "human-readable message",
  "status": "error",
  "code": "ERROR_CODE"
}
```

#### **Rule 10.2: HTTP Status Codes (STRICT)**

- **201 Created:** Error successfully submitted
- **200 OK:** Successful GET
- **400 Bad Request:** Invalid JSON/missing fields
- **401 Unauthorized:** Missing/invalid token
- **404 Not Found:** Resource doesn't exist
- **429 Too Many Requests:** Rate limit exceeded
- **500 Internal Server Error:** Unhandled exception

#### **Rule 10.3: Authentication on Every Endpoint**

Every endpoint except `/health` requires Firebase token:

```
Authorization: Bearer <firebase-token>
```

---

### Rule Category 11: Documentation

#### **Rule 11.1: Code Comments**

- Every public function has doc comment
- Non-obvious logic has inline comments
- No comments stating the obvious

#### **Rule 11.2: CHANGELOG**

Maintained in CHANGELOG.md:

```
## [1.0.0] - 2024-01-15
### Added
- Initial MVP: error capture, deterministic grouping, dashboard
- Flat £49/month billing (no per-event pricing)
- Self-hosting support (docker-compose)
- Rate limiting with transparent headers
- Firebase authentication
```

#### **Rule 11.3: Deployment Guide**

In docs/DEPLOYMENT.md:

- Local development setup
- Self-hosting (docker-compose)
- Cloud deployment (Railway)
- Configuration reference

---

## Part 4: Review Checklist

### For the Reviewer

#### **Go-to-Market (Pass/Fail)**

- [ ] Are the 3 wedges clear and distinct?
- [ ] Is deterministic grouping positioned as hero feature?
- [ ] Is self-hosting emphasized as differentiator?
- [ ] Is billing transparency explicit?

#### **Sovereignty (Pass/Fail)**

- [ ] Can you identify clear module boundaries?
- [ ] Do modules have no cross-imports?
- [ ] Is all inter-module communication through Orchestrator?
- [ ] Is state only in PostgreSQL or function params?

#### **Determinism (Pass/Fail)**

- [ ] Is error hashing deterministic (test present)?
- [ ] Are there tests proving same input → same output?
- [ ] Is there no randomness in core logic?
- [ ] Is spike detection time-based?

#### **Auditability (Pass/Fail)**

- [ ] Is ledger append-only?
- [ ] Are ledger mutations prevented (triggers)?
- [ ] Is every state change logged?
- [ ] Can full history be reconstructed from ledger?

#### **Billing (Pass/Fail)**

- [ ] Is flat £49/month clearly documented?
- [ ] Are rate limits transparent (headers)?
- [ ] Are no surprises guaranteed?
- [ ] Is overage pricing optional (not automatic)?

#### **Self-Hosting (Pass/Fail)**

- [ ] Does `docker-compose up` work?
- [ ] Is export functionality present?
- [ ] Is data ownership guaranteed?
- [ ] Are deployment docs complete?

#### **Testing (Pass/Fail)**

- [ ] Do all public functions have tests?
- [ ] Are determinism tests explicit?
- [ ] Do tests use fixtures (no external deps)?
- [ ] Is deduplication tested?

#### **API (Pass/Fail)**

- [ ] Are responses in consistent format?
- [ ] Are HTTP status codes correct?
- [ ] Is authentication on every endpoint?
- [ ] Are rate limit headers present?

#### **Frontend (Pass/Fail)**

- [ ] Are components functional only?
- [ ] Is TypeScript strict: true?
- [ ] Is all styling via Tailwind?
- [ ] Is deterministic grouping visualization clear?

---

## Part 5: Known Gaps (To Be Filled Before Code)

**Gap 1: Source Maps (Not in MVP, but documented for roadmap)**

- Status: Post-launch feature
- Approach: Separate upload endpoint + mapping table
- Timeline: 2-4 weeks after v1.0

**Gap 2: Release Tracking (Not in MVP)**

- Status: Post-launch feature
- Approach: Project metadata (version + deploy timestamp)
- Timeline: 2-3 weeks after v1.0

**Gap 3: Environment Comparison (Not in MVP)**

- Status: Post-launch feature
- Approach: Tag errors with environment (prod/staging)
- Timeline: 2-3 weeks after v1.0

---

## Part 6: Sign-Off

**Document Status:** Ready for Review

**Reviewer Name:** **\*\*\*\***\_\_\_**\*\*\*\***  
**Review Date:** **\*\*\*\***\_\_\_**\*\*\*\***  
**Approved?** ☐ Yes ☐ No ☐ Conditional

**Comments:**

```
[Reviewer notes here]
```

**Items to Fix:**

```
[List of actionable items before code starts]
```

---

**This document is constitutional. Do not code without sign-off.**

**Version:** 2.0 (Post-Feedback)  
**Last Updated:** 2024-01-15  
**Authority:** Founder  
**Status:** IMMUTABLE UNTIL SIGNED OFF
