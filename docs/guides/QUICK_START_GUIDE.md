# FaultReport: Quick Start Guide for Developers

**Read this first. It tells you exactly what to do, in order.**

---

## 🎯 The Goal

Build a deterministic error tracking system in 3 weeks. Launch on Hacker News.

**Current Status:** 55% complete. Backend mostly done, frontend skeleton, Docker missing.

---

## 📋 What Needs to Be Built (In Order)

### STEP 1: Make Docker Work (TODAY — 1-2 hours)

**Why:** Everything else depends on this. Can't test backend, frontend, or integration without it.

**What to do:**

1. Create `docker-compose.yml` in project root:
```yaml
version: '3.8'

services:
  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: faultreport
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5

  backend:
    build: ./backend
    ports:
      - "8000:8000"
    environment:
      DATABASE_URL: postgresql://postgres:password@postgres:5432/faultreport
      RUST_LOG: debug
      ALLOWED_ORIGINS: localhost:3000,localhost:5173
    depends_on:
      postgres:
        condition: service_healthy
    command: ["faultreport"]

  frontend:
    build: ./frontend
    ports:
      - "3000:3000"
    environment:
      VITE_API_URL: http://backend:8000
    depends_on:
      - backend

volumes:
  postgres_data:
```

2. Create `backend/Dockerfile`:
```dockerfile
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/faultreport /app/
EXPOSE 8000
CMD ["/app/faultreport"]
```

3. Create `frontend/Dockerfile`:
```dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 3000
CMD ["nginx", "-g", "daemon off;"]
```

4. Create `frontend/nginx.conf`:
```nginx
server {
    listen 3000;
    location / {
        root /usr/share/nginx/html;
        try_files $uri $uri/ /index.html;
    }
    location /api {
        proxy_pass http://backend:8000;
    }
}
```

5. Test it:
```bash
docker-compose down -v  # Clean slate
docker-compose up       # Should all start

# In another terminal:
curl http://localhost:8000/api/health  # Should return 200
curl http://localhost:3000              # Should return HTML
```

**Success:** All three services online in < 2 minutes.

---

### STEP 2: Fix Backend 500 Errors (TODAY — 1-2 hours)

**Why:** Benchmark shows all requests return 500. Need to debug.

**What to do:**

```bash
# Check logs
docker-compose logs backend

# Look for errors like:
#   - "migration failed" → schema issue
#   - "connection refused" → postgres not ready
#   - "database error" → sqlx query issue
```

**Common issues:**

1. **Migrations not running:**
   - Check: `docker-compose logs backend | grep "migration"`
   - Fix: Ensure `sqlx migrate run` is called in `main.rs`

2. **Database schema incomplete:**
   - Check: `docker-compose exec postgres psql -U postgres -d faultreport -c "\dt"`
   - Should show: users, projects, errors, ledger, alert_dedup, rate_limit_tracker

3. **Handlers returning 500:**
   - Add debug logging to handlers
   - Check: `docker-compose logs backend` for panic messages
   - Fix: Update error handling in handlers.rs

**Success:** `curl http://localhost:8000/api/health` returns 200.

---

### STEP 3: Implement POST /api/projects Endpoint (TODAY/TOMORROW — 2 hours)

**Why:** Users need a way to create projects and get API keys. This is the critical path.

**What it should do:**
```bash
curl -X POST http://localhost:8000/api/projects \
  -H "Authorization: Bearer some_token" \
  -H "Content-Type: application/json" \
  -d '{"name":"My Project"}'

# Response:
# {
#   "project_id": "uuid",
#   "api_key": "frp_abc123...",
#   "created_at": "2024-01-15T10:00:00Z"
# }
```

**Implementation:**

1. Add handler in `src/handlers.rs`:
```rust
pub async fn create_project(
  pool: web::Data<PgPool>,
  req: web::Json<CreateProjectRequest>,
) -> Result<HttpResponse, FaultReportError> {
  // For now: use hardcoded user_id (mock auth)
  let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")?;

  let (project_id, api_key) = 
    projects::create_project(&pool, user_id, &req.name).await?;

  Ok(HttpResponse::Created().json(json!({
    "project_id": project_id,
    "api_key": api_key,
    "created_at": chrono::Utc::now()
  })))
}
```

2. Add route in `src/orchestrator.rs`:
```rust
.route("/api/projects", web::post().to(handlers::create_project))
```

3. Test it:
```bash
curl -X POST http://localhost:8000/api/projects \
  -H "Content-Type: application/json" \
  -d '{"name":"Test Project"}'

# Should return 201 with project_id and api_key
```

**Success:** Can create a project and get an API key.

---

### STEP 4: Test Error Submission (TODAY/TOMORROW — 30 min)

**Why:** Verify the core flow works end-to-end.

**What to do:**

```bash
# 1. Create project (from STEP 3)
PROJECT_RESPONSE=$(curl -s -X POST http://localhost:8000/api/projects \
  -H "Content-Type: application/json" \
  -d '{"name":"Test"}')

API_KEY=$(echo $PROJECT_RESPONSE | jq -r '.api_key')
PROJECT_ID=$(echo $PROJECT_RESPONSE | jq -r '.project_id')

# 2. Submit error with that API key
curl -X POST http://localhost:8000/api/projects/$PROJECT_ID/errors \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Test error",
    "stack": "at line 1\nat line 2",
    "context": {
      "url": "https://example.com",
      "browser": "Chrome",
      "os": "macOS"
    }
  }'

# Should return 201 with:
# {
#   "id": "uuid",
#   "hash": "abc123...",
#   "was_duplicate": false,
#   "count": 1
# }

# 3. Submit same error again
curl -X POST http://localhost:8000/api/projects/$PROJECT_ID/errors \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"message":"Test error","stack":"at line 1\nat line 2","context":{"url":"https://example.com"}}'

# Should return 201 with:
# {
#   "id": "uuid",  # SAME as before
#   "hash": "abc123...",  # SAME as before
#   "was_duplicate": true,  # KEY: indicates dedup
#   "count": 2  # Incremented!
# }
```

**Success:** Same error → same hash, count increments, deduplication works.

---

### STEP 5: Complete Alert Module + Slack (TOMORROW — 2-3 hours)

**Why:** Spike detection works, but Slack posting is stubbed. Need full implementation.

**What to do:**

1. Update `src/modules/alert.rs`:
```rust
pub async fn post_to_slack(
  webhook_url: &str,
  message: &str,
) -> Result<()> {
  let client = reqwest::Client::new();
  let body = json!({
    "text": "🚨 Error Spike Detected",
    "blocks": [{
      "type": "section",
      "text": {
        "type": "mrkdwn",
        "text": message
      }
    }]
  });

  client
    .post(webhook_url)
    .json(&body)
    .send()
    .await?;

  Ok(())
}
```

2. Update handler to call post_to_slack when spike detected (in `src/handlers.rs`):
```rust
if alert::check_spike(&pool, project_id, &error_hash).await? {
  if alert::should_alert(&pool, project_id, &error_hash).await? {
    let pool_clone = pool.clone();
    let webhook = config.slack_webhook_url.clone();
    tokio::spawn(async move {
      let _ = alert::post_to_slack(
        &webhook,
        "🚨 Error spike detected: More than 10 errors in 5 minutes"
      ).await;
      let _ = alert::record_alert(&pool_clone, project_id, &error_hash).await;
    });
  }
}
```

3. Add to config:
```bash
# .env or docker-compose.yml
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/YOUR/WEBHOOK/URL
```

4. Test it:
   - Create test Slack workspace (slack.com/get-started)
   - Create incoming webhook
   - Submit 11+ errors rapidly
   - Should see Slack message appear

**Success:** Spike detected, Slack message posted.

---

### STEP 6: Integration Tests (TOMORROW/NEXT DAY — 2 hours)

**Why:** Prove everything works end-to-end. Catches regressions.

**What to do:**

Create `tests/e2e_full_flow.rs`:
```rust
#[tokio::test]
async fn test_full_error_submission_flow() {
  // 1. Setup test DB + server
  let pool = setup_test_db().await;
  let client = test_client();

  // 2. Create project
  let project_response = client
    .post("/api/projects")
    .json(&json!({"name": "Test"}))
    .send()
    .await;
  assert_eq!(project_response.status(), 201);
  let api_key = project_response.json::<serde_json::Value>()
    .await["api_key"].as_str().unwrap();

  // 3. Submit error
  let error_response = client
    .post(&format!("/api/projects/{}/errors", project_id))
    .header("Authorization", format!("Bearer {}", api_key))
    .json(&json!({
      "message": "Test error",
      "stack": "line 1\nline 2",
      "context": {"url": "https://example.com"}
    }))
    .send()
    .await;
  assert_eq!(error_response.status(), 201);

  // 4. Verify in DB
  let stored = sqlx::query("SELECT COUNT(*) FROM errors")
    .fetch_one(&pool)
    .await
    .unwrap();
  assert_eq!(stored.get::<i64, _>(0), 1);

  // 5. Submit same error again
  let dup_response = client
    .post(&format!("/api/projects/{}/errors", project_id))
    .header("Authorization", format!("Bearer {}", api_key))
    .json(&json!({
      "message": "Test error",
      "stack": "line 1\nline 2",
      "context": {"url": "https://example.com"}
    }))
    .send()
    .await;
  assert!(dup_response.json::<serde_json::Value>()["was_duplicate"].as_bool());

  // 6. Verify count incremented
  let stored = sqlx::query("SELECT count FROM errors LIMIT 1")
    .fetch_one(&pool)
    .await
    .unwrap();
  assert_eq!(stored.get::<i64, _>(0), 2);
}
```

Run it:
```bash
cd backend
cargo test --test e2e_full_flow
```

**Success:** All tests pass.

---

## 🚀 NOW: Build Frontend (Week 2)

Once backend is solid, build React dashboard.

### STEP 7: Create React Components

**What to build (in order):**

1. **ErrorList.tsx** — Table of errors
2. **ErrorDetail.tsx** — Single error view
3. **Layout.tsx** — Header/sidebar/main
4. **Dashboard.tsx** — Main page
5. **Login.tsx** — Auth
6. **ProjectSetup.tsx** — First-time setup guide

### STEP 8: Wire Up APIs

Connect React components to backend:
- Fetch errors: `GET /api/projects/{id}/errors`
- Get detail: `GET /api/projects/{id}/errors/{id}`
- Create project: `POST /api/projects`

### STEP 9: Test Everything

```bash
# Start everything
docker-compose up

# Test in browser
open http://localhost:3000

# Should see:
# 1. Login page (or dashboard if already signed in)
# 2. Can create project
# 3. Can view errors
# 4. Can click error to see detail
```

---

## ✅ Checklist: Week 1 Complete

- [ ] docker-compose.yml works
- [ ] All 3 services start successfully
- [ ] Backend /api/health returns 200
- [ ] POST /api/projects works
- [ ] Error submission works
- [ ] Deduplication works (same error → count increments)
- [ ] Alert/Slack integration works
- [ ] All integration tests pass
- [ ] Hash determinism proven

---

## ✅ Checklist: Week 2 Complete

- [ ] React dashboard built
- [ ] Can login
- [ ] Can create project
- [ ] Can submit error (via SDK or API)
- [ ] Can view error list
- [ ] Can click error to see detail
- [ ] Can see hash (prove determinism)
- [ ] All tests pass
- [ ] `npm run build` succeeds

---

## ✅ Checklist: Week 3 Complete

- [ ] Deployed to Railway
- [ ] Stripe integration works
- [ ] HN post published
- [ ] First users onboarded

---

## 🆘 When You Get Stuck

### Backend Issue? Check:
```bash
# Logs
docker-compose logs backend

# Database connected?
curl http://localhost:8000/api/health

# Migrations ran?
docker-compose exec postgres psql -U postgres -d faultreport -c "\dt"

# Specific error?
docker-compose logs backend | grep -i "error"
```

### Frontend Issue? Check:
```bash
# Dev server
cd frontend && npm run dev

# Build
cd frontend && npm run build

# Logs in browser console (F12)
```

### Docker Issue? Check:
```bash
# Clean start
docker-compose down -v
docker-compose up

# Rebuild images
docker-compose build --no-cache
docker-compose up
```

---

## 📚 Reference

**Key Files:**
- `docker-compose.yml` — Local dev setup
- `backend/src/main.rs` — Rust entry point
- `backend/src/handlers.rs` — HTTP endpoints
- `backend/src/modules/` — Core business logic
- `frontend/src/main.tsx` — React entry point
- `frontend/src/App.tsx` — Routing

**Key Endpoints:**
- `GET /api/health` — Health check
- `POST /api/projects` — Create project
- `GET /api/projects/{id}/errors` — List errors
- `POST /api/projects/{id}/errors` — Submit error

**Key Tests:**
```bash
cd backend
cargo test --test e2e_*
cargo test --lib  # Unit tests
```

---

## 🎯 The Goal (Again)

**Launch on HN in 3 weeks with:**
- ✅ Deterministic error grouping (core feature)
- ✅ Self-hosting (docker-compose)
- ✅ Predictable billing (£49/month)
- ✅ First users + revenue

**You can do this. Let's go. 🚀**

---

**Last Updated:** 2024-04-06  
**Status:** Ready to build
