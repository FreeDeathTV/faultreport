# Architecture Decision Records (ADR)

**Record of major technical decisions and trade-offs made for FaultReport**

---

## ADR-001: Deterministic Hash-Based Error Grouping

**Status:** ✅ ACCEPTED & LOCKED IN

**Decision:** Use SHA256(message + stack[:10] + url) for deterministic error grouping instead of probabilistic ML-based grouping (like Sentry).

**Rationale:**

- **Sentry's approach:** Uses probabilistic heuristics → same error sometimes groups differently → alert fatigue, manual cleanup
- **Our approach:** Deterministic hash → same error always groups together → zero false positives, no manual work
- **Proof:** Mathematically sound (SHA256 collisions negligible), cryptographically guaranteed, auditable

**Consequences:**

- ✅ No false positive groupings (hero feature)
- ✅ Hash never changes (immutable)
- ✅ Users can verify grouping themselves
- ❌ Can't customize grouping rules (not needed)
- ❌ Can't merge different errors into same group (not needed)

**Trade-offs Made:**

- Chose simplicity (single algorithm) over flexibility (multiple grouping strategies)
- Locked in hash algorithm → cannot change without breaking history

**Implementation:**

```rust
fn compute_hash(message: &str, stack: &str, url: &str) -> String {
  let message_trim = message.trim().replace("\r\n", "\n");
  let stack_frames: Vec<&str> = stack.lines().map(|l| l.trim()).take(10).collect();
  let stack_clean = stack_frames.join("\n");
  let url_clean = url.split('?').next().unwrap_or(url).split('#').next().unwrap_or(url);

  let input = format!("{}\n{}\n{}", message_trim, stack_clean, url_clean);
  let digest = sha256(input.as_bytes());
  hex::encode(digest)
}
```

**Not Included in Hash (Intentional):**

- ❌ Timestamps (allows grouping across time)
- ❌ User IDs (allows grouping across users)
- ❌ Browser/OS versions (focuses on root cause)
- ❌ Custom context (keeps hash stable)

**Future Consideration:** If users need grouping by environment/browser/OS, use secondary indexing (not hash).

---

## ADR-002: Append-Only Ledger for Auditability

**Status:** ✅ ACCEPTED & LOCKED IN

**Decision:** Implement append-only ledger table (events) with database triggers preventing UPDATE/DELETE.

**Rationale:**

- Sentry's approach: Errors table updated in-place (count, timestamps) → can't verify history
- Our approach: Ledger records every state change (new_error, duplicate, alert_sent) → full audit trail
- Compliance: Meets GDPR/HIPAA requirements for immutable audit logs

**Implementation:**

```sql
CREATE TABLE ledger (
  id BIGSERIAL PRIMARY KEY,
  project_id UUID NOT NULL,
  error_id UUID NOT NULL,
  event_type VARCHAR(50) NOT NULL,  -- 'new_error' | 'duplicate'
  data JSONB,
  created_at TIMESTAMP DEFAULT NOW()
);

CREATE TRIGGER ledger_immutable BEFORE UPDATE OR DELETE ON ledger
FOR EACH ROW EXECUTE FUNCTION raise_error('Ledger is immutable');
```

**Consequences:**

- ✅ Full audit trail (all state changes recorded)
- ✅ Compliant with GDPR/HIPAA (immutable)
- ✅ Can reconstruct error history
- ❌ Slightly larger database (ledger grows with every duplicate)
- ❌ Can't delete data (only soft delete via status)

**Query Pattern:**

```sql
-- Get all events for an error
SELECT * FROM ledger WHERE error_id = $1 ORDER BY created_at;

-- Count events by type
SELECT event_type, COUNT(*) FROM ledger WHERE project_id = $1 GROUP BY event_type;
```

---

## ADR-003: Hard Rate Limit (10K/hour) with Explicit Rejection

**Status:** ✅ ACCEPTED & LOCKED IN

**Decision:** Enforce hard cap at 10K errors/hour per project. Reject errors beyond cap with HTTP 429 (do not silently drop).

**Rationale:**

- Sentry's approach: Per-event pricing → surprise £1000+ invoices → unpredictable budgets
- Our approach: Flat £49/month with transparent hard cap → budgets predictable
- Key principle: No silent drops (client sees 429, not success)

**Implementation:**

```rust
pub async fn check_rate_limit(pool: &PgPool, project_id: Uuid) -> Result<bool> {
  let one_hour_ago = Utc::now() - chrono::Duration::hours(1);
  let count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM errors WHERE project_id = $1 AND created_at > $2"
  )
  .bind(project_id)
  .bind(one_hour_ago)
  .fetch_one(pool)
  .await?;

  Ok(count < 10000)
}

// In handler:
if !storage::check_rate_limit(&pool, project_id).await? {
  return Err(FaultReportError::RateLimitExceeded);  // Returns 429
}
```

**Response Headers:**

```
X-RateLimit-Limit: 10000
X-RateLimit-Remaining: 9875
X-RateLimit-Reset: 1705339200
```

**Consequences:**

- ✅ Transparent (client knows exactly how much capacity left)
- ✅ No surprises (hard limit, no throttling)
- ✅ Budget predictable (£49/month, no overages)
- ❌ Users over cap must wait (not auto-throttle)
- ❌ Can't burst past cap (by design)

**Overage Plan (Future):** £99/month option for unlimited (opt-in, not automatic).

---

## ADR-004: Self-Hosting First (docker-compose)

**Status:** ✅ ACCEPTED & LOCKED IN

**Decision:** Make self-hosting the primary deployment method. Cloud hosted is secondary.

**Rationale:**

- Market opportunity: Sentry forces cloud (no self-hosting) → abandoned market for self-hosters
- EU/compliance: GDPR, HIPAA, fintech need data sovereignty → self-hosting is differentiator
- Speed: `docker-compose up` is faster than SaaS signup

**Supported Deployments:**

1. ✅ Self-hosted (docker-compose) — PRIMARY
2. ✅ Cloud (Railway) — SECONDARY
3. ❌ Kubernetes — NOT SUPPORTED (keep simple)
4. ❌ Managed PostgreSQL — NOT REQUIRED (bring your own)

**Design Decisions:**

- All services must work in docker-compose (no external dependencies)
- Environment variables only (no config files)
- Data in PostgreSQL (not managed services)
- No vendor lock-in (can export data anytime)

**Consequences:**

- ✅ Works everywhere (Windows, Mac, Linux)
- ✅ Minimal infrastructure needed
- ✅ Full data control
- ❌ Users must manage own Postgres (slightly more work)
- ❌ No automatic scaling (bring your own k8s if needed)

**Testing Requirement:** docker-compose must work on 5+ platforms before launch.

---

## ADR-005: TypeScript Everywhere (Strict Mode)

**Status:** ✅ ACCEPTED

**Decision:** Use TypeScript in both backend (via Rust types) and frontend (strict mode).

**Rationale:**

- Type safety catches errors at compile time
- Self-documenting code (types are documentation)
- Refactoring confidence (compiler checks all usages)

**Frontend Config:**

```json
{
  "strict": true,
  "noUnusedLocals": true,
  "noUnusedParameters": true,
  "noImplicitAny": true
}
```

**Backend:** Rust's type system is stricter than TypeScript (no opt-out).

**Consequences:**

- ✅ Fewer runtime bugs
- ✅ Better IDE support
- ✅ Easier refactoring
- ❌ Longer initial development (more typing)
- ❌ Learning curve for non-TS developers

---

## ADR-006: Flat Billing (£49/month, No Per-Event Pricing)

**Status:** ✅ ACCEPTED & LOCKED IN

**Decision:** Single flat rate (£49/month) instead of per-event pricing. Unlimited errors, 30-day retention.

**Rationale:**

- Sentry's approach: Per-event ($5-50 per event) → unpredictable costs → users avoid tracking → bad product
- Our approach: Flat rate → use as much as you want → better product
- Cost structure: Server, database, Slack → scales slowly, not linearly with events

**Pricing Tiers (Future, not MVP):**

- Starter: £49/month (1 project, 5 users)
- Pro: £99/month (5 projects, unlimited users) — or overage for rate limit
- Enterprise: Custom (self-hosted support)

**Consequences:**

- ✅ Simple (one plan, not confusing)
- ✅ Transparent (predictable cost)
- ✅ Aligned incentives (we want errors, customer wants tracking)
- ❌ May lose enterprise deals (need custom pricing)
- ❌ High-volume users might saturate

**Rate Limit Tradeoff:** 10K/hour keeps costs reasonable while allowing 864K errors/day.

---

## ADR-007: React (Not Next.js)

**Status:** ✅ ACCEPTED

**Decision:** Use vanilla React 18 + React Router instead of Next.js or Remix.

**Rationale:**

- Simpler architecture (no server-side rendering needed)
- Faster development (no build complexity)
- Easier deployment (static files + API)
- No vendor lock-in (can run anywhere)

**Trade-offs:**

- ✅ Simplicity (React only, no framework)
- ✅ Fast development cycle
- ✅ Easier to understand
- ❌ No SEO benefits (not needed for dashboard)
- ❌ No server-side rendering (not needed)
- ❌ Slightly larger bundle (minimal)

**Setup:**

- Vite (not Webpack)
- Tailwind (not styled-components)
- React Router v6
- TypeScript strict mode

---

## ADR-008: PostgreSQL (Not NoSQL)

**Status:** ✅ ACCEPTED

**Decision:** Use PostgreSQL (relational DB) instead of MongoDB/DynamoDB.

**Rationale:**

- ACID guarantees (data never corrupted)
- JSON support (flexible schema when needed)
- Full-text search (future feature)
- Well-understood performance characteristics
- Cheap/free on Railway

**Schema Design:**

- Errors table (main records)
- Ledger table (immutable audit log)
- Projects table (user's projects)
- Users table (Firebase auth)
- alert_dedup table (spike dedup)

**Consequences:**

- ✅ Reliable (ACID)
- ✅ Simple (standard SQL)
- ✅ Powerful (JSONB, full-text search)
- ❌ Need migrations (not schemaless)
- ❌ Scale-out harder (but not needed for MVP)

---

## ADR-009: Actix-web (Not Warp, Not Rocket)

**Status:** ✅ ACCEPTED

**Decision:** Use Actix-web framework for Rust backend.

**Rationale:**

- High performance (top benchmarks)
- Mature ecosystem (stable, well-tested)
- Async/await (native support)
- Easy middleware system
- Good error handling

**Comparison:**
| Framework | Performance | Maturity | Ease | Choice |
|-----------|-------------|----------|------|--------|
| Actix-web | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ✅ |
| Warp | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Alternative |
| Rocket | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Less mature |
| Axum | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | Too new |

**Consequences:**

- ✅ Fast (1000+ req/sec easily)
- ✅ Mature (proven in production)
- ✅ Handles load well
- ❌ Slightly verbose syntax
- ❌ Learning curve

---

## ADR-010: SQLx (Compile-Time Query Checking)

**Status:** ✅ ACCEPTED

**Decision:** Use sqlx with compile-time query validation instead of sqlc, Diesel, or ORM.

**Rationale:**

- Queries checked at compile time (catch SQL errors before runtime)
- No code generation bloat (direct SQL)
- Deterministic (same query always behaves same way)
- Type-safe (results strongly typed)

**Setup:**

```rust
let row = sqlx::query("SELECT id, hash FROM errors WHERE project_id = $1")
  .bind(project_id)
  .fetch_one(&pool)
  .await?;

let id: Uuid = row.get("id");
let hash: String = row.get("hash");
```

**Consequences:**

- ✅ Type-safe (compile-time checking)
- ✅ No ORM complexity
- ✅ Raw SQL visibility
- ❌ Requires `sqlx-cli` for migrations
- ❌ SQL queries not checked in tests (unless DB running)

---

## ADR-011: Module Isolation (No Cross-Module Imports)

**Status:** ✅ ACCEPTED & LOCKED IN

**Decision:** Enforce module sovereignty: modules never call each other, all communication via Orchestrator.

**Structure:**

```
                [Orchestrator]
                     |
    __________________|__________________
    |                 |                  |
[ErrorCapture]   [Storage]           [Alert]
    (Module A)   (Module B)          (Module C)
```

**No Arrows Between Modules:** ErrorCapture doesn't call Storage, etc.

**Rationale:**

- Clear boundaries (easy to understand)
- Testable (mock each module independently)
- Replaceable (swap implementations)
- Parallelizable (can work on modules in parallel)

**Consequences:**

- ✅ Clean architecture
- ✅ Testable
- ✅ Maintainable
- ❌ Slightly more code (no shared utilities)
- ❌ Orchestrator becomes thin routing layer

---

## ADR-012: Ledger as Source of Truth

**Status:** ✅ ACCEPTED

**Decision:** Ledger is authoritative source of state changes. Errors table is denormalized for fast reads.

**Pattern:**

1. Event happens (new error, duplicate detected, spike alert)
2. Event recorded in ledger (immutable)
3. Errors table updated (for fast queries)
4. Can always rebuild Errors from Ledger

**Example:**

```sql
-- Ledger is authoritative
INSERT INTO ledger (project_id, error_id, event_type) VALUES (...)

-- Errors table is denormalized copy for speed
UPDATE errors SET count = count + 1 WHERE id = ...

-- Can rebuild:
SELECT error_id, COUNT(*) as count FROM ledger GROUP BY error_id
```

**Consequences:**

- ✅ Full audit trail
- ✅ GDPR/HIPAA compliant
- ✅ Can verify data integrity
- ❌ Two-table consistency (need transactions)
- ❌ Slightly slower writes

---

## ADR-013: No Background Threads in MVP

**Status:** ✅ ACCEPTED

**Decision:** For MVP, avoid background jobs/queues. Everything synchronous.

**Rationale:**

- Simpler (no job queue infrastructure)
- Deterministic (easier to test)
- Good enough (small scale)

**Exception:** Async Slack posting (fire-and-forget, doesn't block response).

**Future (Post-MVP):**

- Add job queue (Redis/Bull) if needed
- Background cleanup jobs
- Async batch operations

---

## ADR-014: No Custom Auth (Use Firebase)

**Status:** ✅ ACCEPTED

**Decision:** Use Firebase Auth instead of building custom auth.

**Rationale:**

- Zero infrastructure (Firebase manages everything)
- Standard JWT tokens
- Email + Google OAuth built-in
- Scales automatically

**Consequences:**

- ✅ Zero maintenance
- ✅ Secure (Google maintains)
- ✅ Easy to add Google Sign-In later
- ❌ Firebase vendor lock-in (but easy to replace)
- ❌ Cold start for Firebase calls

**For MVP:** Can mock Firebase tokens (accept any `Authorization: Bearer ...` header).

---

## ADR-015: No Session Replay (Intentional)

**Status:** ✅ DECIDED (NOT INCLUDED IN MVP)

**Decision:** Do NOT include session replay in MVP. Different product category.

**Rationale:**

- Session replay requires:
  - Client-side recording (JavaScript)
  - Video encoding (CPU heavy)
  - Storage (database bloat)
  - Privacy concerns (GDPR/CCPA)
- FaultReport focuses on: deterministic grouping + predictable billing
- Session replay is: LogRocket/Replay feature, not FaultReport

**Consequences:**

- ✅ Simpler product (fewer features to maintain)
- ✅ No privacy/GDPR issues
- ✅ Lower storage costs
- ❌ Can't show user session context (not critical)
- ❌ Different customer expectations

**Recommendation:** If customers ask, point to LogRocket + FaultReport combo.

---

## ADR-016: No Source Map Upload (Intentional)

**Status:** ✅ DECIDED (POST-LAUNCH FEATURE)

**Decision:** Do NOT include source map uploads in MVP. Post-launch feature (Week 2).

**Rationale:**

- MVP focuses on deterministic grouping
- Source maps add complexity (file uploads, storage, mapping)
- Stack traces already useful without source maps (line numbers visible in error)

**Phased Rollout:**

- V1.0 (MVP): No source maps
- V1.1 (Week 2 post-launch): Add upload endpoint
- V1.2 (Week 3): Add source map resolution

---

## ADR-017: No Breadcrumbs in MVP (Intentional)

**Status:** ✅ DECIDED (POST-LAUNCH FEATURE)

**Decision:** Do NOT include breadcrumbs in MVP. Complicates determinism.

**Rationale:**

- Breadcrumbs are order-dependent (user actions in sequence)
- Order changes → hash changes → grouping breaks
- FaultReport prioritizes: deterministic grouping > breadcrumb context
- Sentry has breadcrumbs but grouping is noisy (trade-off)

**Consequence:** Focus on error message + stack trace (sufficient for MVP).

---

## Summary Table

| ADR | Title                 | Status      | Impact        | Locked? |
| --- | --------------------- | ----------- | ------------- | ------- |
| 001 | Deterministic Hashing | ✅ Accepted | HERO FEATURE  | 🔒      |
| 002 | Append-Only Ledger    | ✅ Accepted | Auditability  | 🔒      |
| 003 | Hard Rate Limit       | ✅ Accepted | Billing Trust | 🔒      |
| 004 | Self-Hosting First    | ✅ Accepted | Market Diff   | 🔒      |
| 005 | TypeScript Strict     | ✅ Accepted | Quality       | ✅      |
| 006 | Flat Billing          | ✅ Accepted | GTM           | 🔒      |
| 007 | React (Not Next.js)   | ✅ Accepted | Speed         | ✅      |
| 008 | PostgreSQL            | ✅ Accepted | Reliability   | ✅      |
| 009 | Actix-web             | ✅ Accepted | Performance   | ✅      |
| 010 | SQLx (No ORM)         | ✅ Accepted | Control       | ✅      |
| 011 | Module Isolation      | ✅ Accepted | Architecture  | 🔒      |
| 012 | Ledger as Source      | ✅ Accepted | Integrity     | ✅      |
| 013 | No Background Jobs    | ✅ Accepted | Simplicity    | ✅      |
| 014 | Firebase Auth         | ✅ Accepted | Convenience   | ✅      |
| 015 | No Session Replay     | ✅ Decided  | Scope         | ✅      |
| 016 | No Source Maps (MVP)  | ✅ Decided  | Timeline      | ✅      |
| 017 | No Breadcrumbs (MVP)  | ✅ Decided  | Determinism   | ✅      |

**🔒 = Cannot change without major impact**

---

**Document Status:** IMMUTABLE (Architecture foundation)  
**Last Updated:** 2024-04-06  
**Author:** FaultReport Architecture Team  
**Review Cadence:** Annual (or before major changes)
