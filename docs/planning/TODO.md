# FaultReport: Master TODO (Revised)

**Updated Project Status: 55% Complete**

Last Updated: 2024-04-06  
Current Phase: Backend Completion + Frontend Kickoff  
Timeline: Target HN Launch within 2-3 weeks

---

## 🚨 IMMUTABLE CONSTRAINTS (DO NOT CHANGE)

- **Hash Algorithm**: SHA256(message_trim + stack[:10 frames] + url_clean) — LOCKED IN
- **Rate Limiting**: Hard cap 10K errors/hour per project (reject with 429 after)
- **Deduplication**: Same hash = same group (deterministic, never merges different errors)
- **Ledger**: Append-only, immutable once written (triggers enforce)
- **Docker**: `docker-compose up` must work on macOS, Linux, Windows (all architectures)

**BREAKING ANY OF THESE = PRODUCT FAILURE**

---

# PHASE 1: BACKEND COMPLETION (Week 1)

## Current Status

- ✅ Hash algorithm complete + tested
- ✅ Database schema + migrations ready
- ✅ Storage module (persist, dedup, rate limits) working
- ✅ Error capture (validation, normalization) done
- ✅ Alert module (spike detection + Slack posting implemented)
- ✅ API key system + Project creation endpoint implemented
- ✅ Docker infrastructure present and verified locally
- ⚠️ Integration tests: harness & some integration tests added (more e2e needed)

## BLOCK 1A: Docker & Local Development Setup (CRITICAL — Do First)

### Create docker-compose.yml

```bash
# Location: /project/docker-compose.yml
# Must include:
# - postgres:15-alpine with health check
# - backend (Rust) with migrations auto-run
# - frontend (React) on port 3000
# - All services must start in < 2 minutes
# - All services must be healthy after startup

# Test:
docker-compose down -v  # Clean slate
docker-compose up       # Should complete with all healthy
curl http://localhost:8000/api/health
curl http://localhost:3000
# Both should respond with 200
```

**Subtasks:**

- [x] Write docker-compose.yml with postgres, backend, frontend services
- [x] Add health checks for all services
- [x] Test on your local machine: `docker-compose up` and verify all 3 services start
- [x] Test shutdown: `docker-compose down` cleans up data properly

**Files to Create/Edit:**

- [x] Create: `docker-compose.yml`
- [x] Edit: `backend/Dockerfile` (create if missing)
- [x] Edit: `frontend/Dockerfile` (create if missing)

### Create Backend Dockerfile

```dockerfile
# Multi-stage build
# Stage 1: Builder (cargo build --release)
# Stage 2: Runtime (copy binary only, minimal base image)
# Expose port 8000
# Run: ./faultreport
```

**Subtasks:**

- [x] Create `backend/Dockerfile` with multi-stage build
- [x] Test build: `docker build -t faultreport:backend .`
- [x] Verify binary size < 50MB

### Create Frontend Dockerfile

```dockerfile
# Multi-stage build
# Stage 1: Node.js builder (npm install, npm run build)
# Stage 2: Nginx (serve dist/ + proxy /api)
# Expose port 3000
```

**Subtasks:**

- [x] Create `frontend/Dockerfile` with multi-stage build
- [x] Create `frontend/nginx.conf` (serve React, proxy /api to backend:8000)
- [x] Test build: `docker build -t faultreport:frontend .`

**Validation Checklist:**

- [x] `docker-compose up` succeeds (verified locally)
- [x] All services healthy within 2 minutes (verified locally)
- [x] `curl http://localhost:8000/api/health` returns 200 with database status
- [x] `curl http://localhost:3000` returns React HTML
- [x] Backend logs show migrations applied

---

## BLOCK 1B: Project Creation Endpoint (CRITICAL)

### Create POST /api/projects Endpoint

Users need a way to create projects and get API keys.

**What it does:**

1. User submits: `POST /api/projects` with `{ name: "My Project" }`
2. Backend generates unique API key
3. Backend hashes + stores in DB
4. Returns: `{ project_id, api_key }` (key shown only once)

**Implementation:**

- [x] Create `src/handlers/projects.rs` with:

  ```rust
  pub async fn create_project(
    pool: web::Data<PgPool>,
    user_id: Uuid,  // from Firebase token
    req: web::Json<CreateProjectRequest>,
  ) -> Result<HttpResponse>
  ```

  - Note: current implementation uses `ApiKeyCache` for auth validation; creating projects is implemented in `src/handlers.rs`.

- [x] Update `src/orchestrator.rs`:

  ```rust
  .route("/api/projects", web::post().to(handlers::create_project))
  ```

- [ ] Test endpoint:
  ```bash
  curl -X POST http://localhost:8000/api/projects \
    -H "Authorization: Bearer $(firebase_token)" \
    -H "Content-Type: application/json" \
    -d '{"name":"My First Project"}'
  # Should return 201 with { project_id, api_key, created_at }
  ```

**Subtasks:**

- [ ] Add Firebase token verification middleware
- [ ] Implement create_project handler
- [ ] Update orchestrator routes
- [ ] Test with real Firebase token (or mock for now)
- [ ] Verify API key is unique + properly hashed in DB

### Verification Test

```bash
# 1. Create project
PROJECT_RESPONSE=$(curl -s -X POST http://localhost:8000/api/projects \
  -H "Authorization: Bearer TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"Test"}')

# 2. Extract API key and project ID
API_KEY=$(echo $PROJECT_RESPONSE | jq -r '.api_key')
PROJECT_ID=$(echo $PROJECT_RESPONSE | jq -r '.project_id')

# 3. Submit error with that API key
curl -X POST http://localhost:8000/api/projects/$PROJECT_ID/errors \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Test error",
    "stack": "at line 1",
    "context": { "url": "https://example.com" }
  }'

# Should return 201 with error_id and hash
```

---

## BLOCK 1C: Firebase Authentication Middleware

Users must be authenticated before creating projects. Add middleware to verify Firebase tokens.

**What it does:**

1. Extract `Authorization: Bearer <token>` from request
2. Call Firebase Admin SDK to verify token
3. Extract `uid` from token

---

## 🔧 Stub Remediation Links

The following detailed issue skeletons and remediation notes were added under `docs/stub_issues/` to track completing stubbed functionality:

- [Alerting / Slack integration](docs/stub_issues/alerting.md)
- [Replace test stubs with real tests](docs/stub_issues/testing.md)
- [Frontend auth: replace Firebase stub](docs/stub_issues/frontend_auth.md)
- [Integration test harness (Docker Compose + migrations)](docs/stub_issues/integration_harness.md)

See `STUBS_TODO.md` for a summary of completed work and next steps. 4. Pass `user_id` to handler

**Implementation:**

- [x] Create `src/middleware/auth.rs` (implemented at `backend/src/middleware/auth.rs`):

```rust
// Helper: pub async fn require_firebase_auth(req: &HttpRequest) -> Result<Uuid>
```

- [ ] Add to handlers that need auth (create project, list projects, etc.):

```rust
let user_id = require_firebase_auth(&req)?;
```

**Subtasks:**

- [ ] Initialize Firebase Admin SDK in `src/auth/firebase.rs`
- [ ] Implement token verification
- [ ] Test: verify a valid Firebase token works
- [ ] Test: reject invalid/missing tokens with 401

**For now (MVP):** Can use a mock Firebase verifier that accepts any `Authorization: Bearer ...` header.

---

## BLOCK 1D: Complete Alert Module (Slack Posting)

Alert module has spike detection, but Slack posting is stubbed.

**What it does:**

1. When error count > 10 in last 5 minutes → spike detected
2. Check if alert already sent in last 5 minutes (dedup)
3. If new spike → post to Slack
4. Include: error message, count, link to dashboard

**Implementation:**

- [x] Update `src/modules/alert.rs`:

  ```rust
  pub async fn post_to_slack(
    webhook_url: &str,
    message: &str,
  ) -> Result<()>
  ```

- [x] Create Slack message format:

  ```json
  {
    "text": "🚨 Error Spike Detected",
    "blocks": [
      {
        "type": "section",
        "text": {
          "type": "mrkdwn",
          "text": "*Error Spike*\nMessage: Cannot read property 'x' of undefined\nCount: 42 in 5 minutes\n<https://dashboard.faultreport.io/errors/hash123|View Details>"
        }
      }
    ]
  }
  ```

- [x] Add environment variable: `SLACK_WEBHOOK_URL` (config present; can be set in env/docker-compose)

- [x] Test: in-repo webhook sink integration test added (`tests/integration/post_slack_integration.rs`)

**Subtasks:**

- [x] Implement `post_to_slack()` using reqwest (helper implemented)
- [x] Add Slack webhook URL to config
- [ ] Test with real Slack webhook (create test workspace)
- [x] Verify message formatting via integration test sink

---

## BLOCK 1E: Integration Tests (Prove Everything Works)

Unit tests exist, but integration tests are stubbed. Create end-to-end tests.

### Test 1: Full Error Submission Flow

```rust
// tests/e2e_error_submission.rs
#[tokio::test]
async fn test_submit_error_end_to_end() {
  // 1. Start server
  // 2. Create project via API
  // 3. Submit error with API key
  // 4. Verify error appears in DB
  // 5. Verify ledger entry created
  // 6. Verify hash is correct
}
```

**Subtasks:**

- [x] Create `tests/e2e_error_submission.rs` with full flow test (basic flow implemented; runs when `TEST_DATABASE_URL` is set)
- [ ] Create `tests/e2e_deduplication.rs` (same error twice → count=2)
- [ ] Create `tests/e2e_rate_limiting.rs` (submit 10K+ errors, verify 429)
- [ ] Create `tests/e2e_alert_slack.rs` (spike triggers Slack) — integration harness and test-sink present (`tests/docker-compose.test.yml`, `tests/integration/post_slack_integration.rs`)

### Test 2: Determinism Validation

Prove hash is deterministic across runs.

**Subtasks:**

- [ ] Create `tests/determinism_proof.rs`
- [ ] Test: Submit same error 100 times, verify same hash every time
- [ ] Test: Different errors → different hashes

### Test 3: API Key Verification

**Subtasks:**

- [ ] Create `tests/api_key_validation.rs`
- [ ] Test: Valid API key → error accepted (201)
- [ ] Test: Invalid API key → error rejected (401)
- [ ] Test: Rotated key works, old key fails

**Run Tests:**

```bash
cd backend
cargo test --test e2e_*  # Should all pass
```

---

## BLOCK 1F: Fix Critical Bugs (As Found)

From benchmark results, all requests return 500. Debug and fix.

**Likely causes:**

- [ ] Database migrations not running on startup
- [ ] Database connection pool misconfigured
- [ ] Handler error response formatting
- [ ] Missing environment variables

**Debug process:**

```bash
docker-compose logs backend
# Look for error messages
# Common issues:
#   - "connection refused" (postgres not ready)
#   - "migration failed" (schema issue)
#   - "database error" (sqlx query issue)
```

**Subtasks:**

- [ ] Check database connectivity: `docker-compose exec backend curl http://localhost:8000/api/health`
- [ ] Verify migrations run: `docker-compose logs postgres` should show schema created
- [ ] Test handler directly: add debug logging to handlers
- [ ] Fix any compilation warnings: `cargo clippy --all-targets`

---

## BLOCK 1 Success Criteria ✅

When ALL of these pass, Block 1 is DONE:

- [ ] `docker-compose up` starts all 3 services successfully
- [ ] All services healthy within 2 minutes
- [ ] `GET /api/health` returns 200 with database status
- [ ] `POST /api/projects` creates a new project and returns API key
- [ ] `POST /api/projects/{id}/errors` accepts error with valid API key
- [ ] Invalid API key rejected with 401
- [ ] Same error submitted twice → count increments (no duplicate record)
- [ ] Error spike (10+ in 5 min) triggers Slack alert
- [ ] All integration tests pass
- [ ] Hash determinism proven (same error → same hash)
- [ ] Rate limit enforced (10K/hour cap)

---

# PHASE 2: FRONTEND BUILD (Week 2)

## Current Status

- ✅ TypeScript config (strict: true)
- ✅ Tailwind config
- ✅ Client API wrapper (basic)
- 🔴 React components missing (ErrorList, ErrorDetail, Layout, etc.)
- 🔴 Authentication flow missing
- 🔴 Dashboard pages missing
- 🔴 Routing missing

## BLOCK 2A: React Component Foundation

### Create Core Components

#### 1. Layout.tsx (Main Shell)

```tsx
// src/components/Layout.tsx
// - Header with logo
// - Sidebar (projects dropdown)
// - Main content area
// - Footer (optional)
```

**Subtasks:**

- [ ] Create Layout component with header/sidebar/content
- [ ] Add project dropdown (mock data for now)
- [ ] Add logout button
- [ ] Style with Tailwind (no custom CSS)

#### 2. ErrorList.tsx (Error Table)

```tsx
// src/components/ErrorList.tsx
// - Table of errors
// - Columns: Message, Count, First Seen, Last Seen, Hash
// - Click row to show detail
// - Pagination (20 per page)
// - Show "Deterministic Grouping" badge
```

**Subtasks:**

- [ ] Create table component
- [ ] Implement pagination
- [ ] Add click handler to show detail
- [ ] Show hash (prove determinism)
- [ ] Loading state (skeleton screen)

#### 3. ErrorDetail.tsx (Single Error View)

```tsx
// src/components/ErrorDetail.tsx
// - Full error message
// - Stack trace (formatted code blocks)
// - Context (URL, browser, OS, user_id, custom)
// - Count + first/last seen
// - Hash (for verification)
// - List of recent occurrences
```

**Subtasks:**

- [ ] Create detail component
- [ ] Format stack trace as code
- [ ] Display JSON context
- [ ] Show hash clearly
- [ ] Add back button

#### 4. EmptyState.tsx (No Errors Yet)

```tsx
// src/components/EmptyState.tsx
// - Show when project has no errors
// - Display SDK setup instructions
// - Copy-to-clipboard for API key
// - Link to docs
```

**Subtasks:**

- [ ] Create empty state component
- [ ] Display API key from props
- [ ] Add copy button
- [ ] Show example code

#### 5. ProjectSelector.tsx (Project Dropdown)

```tsx
// src/components/ProjectSelector.tsx
// - Dropdown of user's projects
// - "+ New Project" button
// - Click to switch project
```

**Subtasks:**

- [ ] Create dropdown component
- [ ] Implement project switching
- [ ] Add new project button
- [ ] Fetch projects on mount

### Type Definitions

- [ ] Create `src/types/index.ts` with all TypeScript interfaces:
  ```typescript
  interface ErrorRecord {
    id: string;
    hash: string;
    message: string;
    count: number;
    first_seen_at: string;
    last_seen_at: string;
  }
  ```

---

## BLOCK 2B: Pages & Routing

### Create Pages

#### 1. Login.tsx

```tsx
// src/pages/Login.tsx
// - Email + password form
// - Sign in button
// - Or sign up link
// - Integrate Firebase Auth
```

**Subtasks:**

- [ ] Create login form
- [ ] Wire Firebase signInWithEmailAndPassword()
- [ ] Handle auth errors
- [ ] Redirect to dashboard on success
- [ ] Add "Sign Up" link

#### 2. Dashboard.tsx

```tsx
// src/pages/Dashboard.tsx
// - Fetch errors for selected project
// - Show ErrorList
// - Show ErrorDetail in modal or split view
// - Pagination
```

**Subtasks:**

- [ ] Create dashboard page
- [ ] Fetch errors on mount
- [ ] Handle click on error row (show detail)
- [ ] Implement pagination
- [ ] Add loading state

#### 3. ProjectSetup.tsx (First Time)

```tsx
// src/pages/ProjectSetup.tsx
// - Show if project created but no errors yet
// - Display API key (from URL param)
// - Show SDK installation instructions
// - Copy button for key
```

**Subtasks:**

- [ ] Create setup page
- [ ] Fetch project details
- [ ] Display API key + instructions
- [ ] Add copy-to-clipboard
- [ ] Auto-redirect to dashboard when first error arrives

#### 4. Settings.tsx (Project Settings)

```tsx
// src/pages/Settings.tsx
// - Project name
// - API key (redacted)
// - Rotate API key button
// - Confirm dialog
```

**Subtasks:**

- [ ] Create settings page
- [ ] Show project name
- [ ] Display redacted API key (show last 4 chars only)
- [ ] Implement rotate key functionality
- [ ] Add confirmation dialog

### Routing

- [ ] Create `src/App.tsx`:
  ```typescript
  /login → Login page
  /dashboard → Dashboard (protected)
  /projects/:id/setup → Project setup
  /projects/:id/settings → Settings
  / → Redirect to /dashboard
  ```

**Subtasks:**

- [ ] Create App.tsx with React Router
- [ ] Add protected route wrapper (redirect to login if not auth)
- [ ] Implement routing logic
- [ ] Test all routes work

---

## BLOCK 2C: API Integration

### Update API Client (src/api/client.ts)

Already exists but needs completion:

```typescript
// Implement these functions:
export async function listErrors(
  projectId: string,
  page: number,
): Promise<ErrorResponse>;
export async function getError(
  projectId: string,
  errorId: string,
): Promise<ErrorDetail>;
export async function createProject(
  name: string,
): Promise<CreateProjectResponse>;
export async function rotateApiKey(
  projectId: string,
): Promise<RotateKeyResponse>;
export async function getProject(projectId: string): Promise<ProjectResponse>;
```

**Subtasks:**

- [ ] Implement all API functions
- [ ] Add error handling (401 → redirect to login)
- [ ] Add loading states
- [ ] Test with real backend API

### Firebase Auth Setup

- [ ] Create `src/auth/firebase.ts`:

  ```typescript
  initializeFirebase();
  signInWithEmail(email, password);
  signOut();
  getCurrentUser();
  ```

- [ ] Update handlers to use Firebase auth
- [ ] Test sign in / sign out flow

---

## BLOCK 2D: Component Tests

Test that components render correctly.

**Subtasks:**

- [ ] Create `src/__tests__/ErrorList.test.tsx`
  - [ ] Test renders table
  - [ ] Test pagination
  - [ ] Test click handler
- [ ] Create `src/__tests__/ErrorDetail.test.tsx`
  - [ ] Test displays error
  - [ ] Test formats stack trace
- [ ] Create `src/__tests__/Dashboard.test.tsx`
  - [ ] Test fetches errors
  - [ ] Test loads ErrorList
  - [ ] Test loading state

**Run Tests:**

```bash
cd frontend
npm run test  # Should all pass
```

---

## BLOCK 2E: Build & Docker

### Build Frontend

```bash
npm run build
# Should create dist/ folder with optimized assets
```

**Subtasks:**

- [ ] Test build succeeds
- [ ] Verify dist/ has HTML + JS bundles
- [ ] Test `npm run preview` serves the build

### Frontend Dockerfile

Already designed, just verify:

- [ ] Multi-stage build works
- [ ] nginx.conf proxies /api correctly
- [ ] Docker build succeeds
- [ ] Docker container runs on port 3000

---

## BLOCK 2 Success Criteria ✅

When ALL of these pass, Block 2 is DONE:

- [ ] Login page works (Firebase or mock)
- [ ] Can sign in and reach dashboard
- [ ] Dashboard fetches errors from backend
- [ ] Error list displays with pagination
- [ ] Click error shows detail view
- [ ] ErrorDetail shows hash (prove determinism)
- [ ] Empty state shown when no errors
- [ ] Project setup page guides SDK installation
- [ ] Settings page allows API key rotation
- [ ] All components render without errors
- [ ] All tests pass
- [ ] Implement ESLint `no-console` rule to prevent sensitive data leaks
- [ ] `npm run build` succeeds
- [ ] `docker build -t frontend .` succeeds
- [ ] `docker-compose up` starts frontend on port 3000
- [ ] Can interact with dashboard (create project, submit error, see it appear)

---

# PHASE 3: DEPLOYMENT & LAUNCH (Week 3)

## BLOCK 3A: Self-Hosting Validation

Ensure docker-compose works on all platforms.

**Platforms to test:**

- [ ] macOS x86_64: `docker-compose up` and verify services online
- [ ] macOS ARM64 (Apple Silicon): Same
- [ ] Linux x86_64: Same
- [ ] Windows 10/11 WSL2: Same
- [ ] Windows 10/11 Docker Desktop: Same

**For each:**

- [ ] Services start in < 2 minutes
- [ ] `/api/health` returns 200
- [ ] Can create project
- [ ] Can submit error
- [ ] Error appears in dashboard
- [ ] Can export data as JSON

---

## BLOCK 3B: Stripe Integration

Add billing (£49/month flat).

**Subtasks:**

- [ ] Create Stripe account
- [ ] Create product + price (£49/month)
- [ ] Implement checkout flow
- [ ] Handle webhooks (subscription created/updated/canceled)
- [ ] Update user table with subscription_id
- [ ] Gate features behind subscription check
- [ ] Create billing page in dashboard

---

## BLOCK 3C: Deployment (Railway)

Deploy to production.

**Subtasks:**

- [ ] Create Railway account
- [ ] Connect GitHub repo
- [ ] Set up environment variables (DATABASE_URL, FIREBASE_KEY, STRIPE_KEY, etc.)
- [ ] Deploy backend
- [ ] Deploy frontend
- [ ] Verify both online and working
- [ ] Set up custom domain (optional)
- [ ] Set up monitoring/logging

---

## BLOCK 3D: Documentation

Create guides for users.

**Subtasks:**

- [ ] Update README.md with quick start
- [ ] Create docs/SELF_HOSTING.md (docker-compose guide)
- [ ] Create docs/BILLING.md (pricing + rate limits)
- [ ] Create docs/DEPLOYMENT.md (Railway deployment)
- [ ] Create CHANGELOG.md (v1.0.0 release notes)
- [ ] Create docs/API.md (API reference)

---

## BLOCK 3E: Launch

**Subtasks:**

- [ ] Write HN post (title: "FaultReport – Deterministic Error Tracking (Sentry Alternative)")
- [ ] Post to HN at optimal time (morning US Pacific)
- [ ] Monitor comments and respond
- [ ] Share on Twitter/Reddit (optional)
- [ ] Send to relevant communities

---

## BLOCK 3 Success Criteria ✅

- [ ] docker-compose works on all major platforms
- [ ] Stripe billing integrated and tested
- [ ] Deployed to Railway
- [ ] All documentation complete
- [ ] HN launch post live
- [ ] First users onboarded
- [ ] First paying customer

---

# QUICK REFERENCE

## Essential Commands

```bash
# Start everything
docker-compose down -v && docker-compose up

# Backend only
cd backend && cargo run

# Frontend only
cd frontend && npm run dev

# Run tests
cd backend && cargo test
cd frontend && npm run test

# Build for production
cd backend && cargo build --release
cd frontend && npm run build

# Check logs
docker-compose logs backend
docker-compose logs frontend
docker-compose logs postgres
```

## Environment Variables Required

```bash
# Backend (.env or docker-compose)
DATABASE_URL=postgresql://postgres:password@postgres:5432/faultreport
RUST_LOG=debug
ALLOWED_ORIGINS=localhost:3000,localhost:5173
FIREBASE_PROJECT_ID=your_firebase_project
SLACK_WEBHOOK_URL=https://hooks.slack.com/...
STRIPE_SECRET_KEY=sk_...

# Frontend (.env.local)
VITE_API_URL=http://localhost:8000
VITE_FIREBASE_CONFIG={...}
```

## File Structure Checklist

```
faultreport/
├── backend/
│   ├── src/
│   │   ├── main.rs ✅
│   │   ├── lib.rs ✅
│   │   ├── config.rs ✅
│   │   ├── db.rs ✅
│   │   ├── error.rs ✅
│   │   ├── handlers.rs ✅ (needs project endpoint)
│   │   ├── orchestrator.rs ✅
│   │   ├── middleware.rs ✅
│   │   ├── modules/
│   │   │   ├── error_capture.rs ✅
│   │   │   ├── storage.rs ✅
│   │   │   ├── alert.rs ⚠️ (needs Slack)
│   │   │   └── projects.rs ✅
│   │   └── auth/
│   │       └── cache.rs ✅
│   ├── migrations/
│   │   ├── 001_initial_schema.sql ✅
│   │   ├── 002_ledger_immutable.sql ✅
│   │   └── 003_indexes.sql ✅
│   ├── tests/
│   │   ├── bench_test.rs ✅ (needs DB debug)
│   │   └── integration_test.rs 🔴 (needs implementation)
│   ├── Dockerfile 🔴 (needs creation)
│   ├── Cargo.toml ✅
│   └── .dockerignore 🔴
│
├── frontend/
│   ├── src/
│   │   ├── main.tsx ✅
│   │   ├── App.tsx 🔴 (needs routing)
│   │   ├── pages/
│   │   │   ├── Login.tsx 🔴
│   │   │   ├── Dashboard.tsx 🔴
│   │   │   ├── ProjectSetup.tsx 🔴
│   │   │   └── Settings.tsx 🔴
│   │   ├── components/
│   │   │   ├── Layout.tsx 🔴
│   │   │   ├── ErrorList.tsx 🔴
│   │   │   ├── ErrorDetail.tsx 🔴
│   │   │   ├── EmptyState.tsx 🔴
│   │   │   └── ProjectSelector.tsx 🔴
│   │   ├── api/
│   │   │   └── client.ts ✅ (needs completion)
│   │   ├── auth/
│   │   │   └── firebase.ts 🔴 (needs implementation)
│   │   ├── types/
│   │   │   └── index.ts 🔴 (needs creation)
│   │   └── index.css ✅
│   ├── Dockerfile 🔴
│   ├── nginx.conf 🔴
│   ├── package.json ✅
│   ├── tsconfig.json ✅
│   ├── vite.config.ts ✅
│   ├── tailwind.config.js ✅
│   └── .dockerignore 🔴
│
├── docker-compose.yml 🔴 (CRITICAL)
├── Cargo.toml ✅
├── README.md ✅
├── ARCHITECTURE.md ✅
├── structure.md ✅
└── TODO_MASTER.md (THIS FILE)
```

---

# NEXT STEPS

## Right Now (Today)

**Priority 1 (Do first):**

1. Create docker-compose.yml (30 min)
2. Create backend Dockerfile (15 min)
3. Test: `docker-compose up` starts successfully
4. Debug any 500 errors in backend

**Priority 2 (After docker works):** 5. Implement POST /api/projects endpoint (1 hour) 6. Test creating a project via API 7. Test submitting error with API key

## This Week

- Complete all of PHASE 1 (Backend)
- Get docker-compose working on all platforms
- Pass all integration tests
- Have Slack alerts working

## Next Week

- Complete all of PHASE 2 (Frontend)
- Build React dashboard
- Wire up auth
- Verify end-to-end flow (create account → create project → submit error → see dashboard)

## Week After

- Deploy to Railway
- Stripe integration
- Launch on HN

---

# Success Metrics

| Milestone             | Target  | Status |
| --------------------- | ------- | ------ |
| Docker working        | 30 min  | 🔴     |
| Backend complete      | 1 week  | 🟡     |
| Frontend complete     | 2 weeks | 🔴     |
| Deployed to prod      | 3 weeks | 🔴     |
| HN launch             | 3 weeks | 🔴     |
| First paying customer | 4 weeks | 🔴     |

---

# Remember

- **Be ruthless about scope** — cut features if they block progress
- **Test early, test often** — especially determinism and dedup
- **Self-hosting is a feature** — docker-compose must work
- **Move fast** — 3 weeks to launch is tight

**Let's build this. 🚀**

---

**Document Status:** IMMUTABLE (New Master Plan)  
**Author:** FaultReport Team  
**Version:** 2.0 (Revised)  
**Last Updated:** 2024-04-06
