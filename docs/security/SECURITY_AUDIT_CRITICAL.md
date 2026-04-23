# 🔒 Security Audit — Critical & High Priority Issues

> **Status:** Ready for Implementation  
> **Audit Date:** 2026-04-23  
> **Auditor:** Pre-deployment security review  
> **Action Required:** Resolve all 🔴 Critical and 🟠 High issues before deploying to production or publishing to GitHub.

---

## 🔴 Critical Issues (MUST FIX Before Deployment)

### CRIT-001: Weak API Key Generation

| Field                   | Details                                                                                                                                                                                                                                                                                                                                                                                                                       |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Severity**            | 🔴 Critical                                                                                                                                                                                                                                                                                                                                                                                                                   |
| **File**                | `backend/src/modules/projects.rs` (lines 9–11)                                                                                                                                                                                                                                                                                                                                                                                |
| **Problem**             | API keys are generated using `hex::encode(Uuid::new_v4().as_bytes())`. This yields only **16 bytes of entropy** encoded as hex. UUID v4 uses predictable patterns and is not suitable for cryptographically secure API keys.                                                                                                                                                                                                  |
| **Impact**              | Attackers can brute-force or guess API keys, gaining unauthorized access to any project's error data.                                                                                                                                                                                                                                                                                                                         |
| **Current Code**        | `rust\npub fn generate_api_key() -> String {\n    let key = hex::encode(Uuid::new_v4().as_bytes());\n    format!("frp_{}", &key[0..32])\n}\n`                                                                                                                                                                                                                                                                                 |
| **Fix**                 | Replace with the existing secure implementation in `backend/src/api_key.rs` which uses `OsRng` with **192 bits of entropy** and base62 encoding: \n`rust\nuse rand::RngCore;\nuse rand::rngs::OsRng;\n\npub fn generate_api_key() -> String {\n    let mut rng = OsRng;\n    let mut bytes = [0u8; 24];\n    rng.fill_bytes(&mut bytes);\n    let base62 = encode_base62(&bytes);\n    format!("frp_{}", &base62[..32])\n}\n` |
| **Acceptance Criteria** | - [ ] `generate_api_key()` uses `OsRng` not `Uuid::new_v4()` \n- [ ] At least 192 bits of entropy \n- [ ] All existing tests pass                                                                                                                                                                                                                                                                                             |

---

### CRIT-002: Authentication Bypass in Production Code

| Field                   | Details                                                                                                                                                                                                                                                                                        |
| ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --- | ----------------------------- | --- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --- | ----------------------------------------- |
| **Severity**            | 🔴 Critical                                                                                                                                                                                                                                                                                    |
| **File**                | `backend/src/middleware/auth.rs` (lines 17–35)                                                                                                                                                                                                                                                 |
| **Problem**             | Three bypass mechanisms exist in the **production authentication middleware**: \n1. `DISABLE_FIREBASE_AUTH=1` env var completely disables auth \n2. `MOCK_USER_ID` env var allows impersonating any user \n3. `Bearer mock:<uuid>` tokens are accepted without any verification                |
| **Impact**              | Anyone with knowledge of these env vars or token formats can completely bypass Firebase authentication, create projects, and access any data.                                                                                                                                                  |
| **Current Code**        | `rust\nif env::var("DISABLE_FIREBASE_AUTH")... {\n    if let Ok(u) = env::var("MOCK_USER_ID") {\n        return Uuid::parse_str(&u)...;\n    }\n    return Ok(Uuid::nil());\n}\nif let Some(s) = token.strip_prefix("mock:") {\n    return Uuid::parse_str(s)...;\n}\n`                        |
| **Fix**                 | Remove **all** mock/bypass logic from `require_firebase_auth()`. Move mock helpers to `#[cfg(test)]` test fixtures only: \n```rust\npub async fn require_firebase_auth(req: &HttpRequest) -> Result<Uuid, FaultReportError> {\n let token = req.headers()\n .get("Authorization")\n .and_then( | h   | h.to_str().ok())\n .and_then( | h   | h.strip_prefix("Bearer "))\n .ok_or(FaultReportError::Unauthorized)?;\n \n let uid = firebase::verify_firebase_token(token).await?;\n let digest = Sha256::digest(uid.as_bytes());\n let mut bytes = [0u8; 16];\n bytes.copy_from_slice(&digest[0..16]);\n Ok(Uuid::from_slice(&bytes).map_err( | \_  | FaultReportError::Unauthorized)?)\n}\n``` |
| **Acceptance Criteria** | - [ ] No env-var bypasses remain in production code \n- [ ] `mock:` prefix tokens are rejected with `401 Unauthorized` \n- [ ] Mock auth remains available in `#[cfg(test)]` modules only                                                                                                      |

---

### CRIT-003: Firebase JWT Validation Gaps

| Field                   | Details                                                                                                                                                                                                                                                                                                           |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --- | --------------------------------------------------------------------------------------------------- | --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --- | ------------------------------------------------------------------------ |
| **Severity**            | 🔴 Critical                                                                                                                                                                                                                                                                                                       |
| **File**                | `backend/src/auth/firebase.rs` (lines 55–90)                                                                                                                                                                                                                                                                      |
| **Problem**             | \n1. Audience (`aud`) and issuer (`iss`) validation is **skipped entirely** if `FIREBASE_PROJECT_ID` is not set. \n2. Uses `validate_exp = false` with manual expiration checking instead of library validation. \n3. No validation of `sub` claim presence.                                                      |
| **Impact**              | Tokens from other Firebase projects, expired tokens, or malformed JWTs could be accepted as valid.                                                                                                                                                                                                                |
| **Current Code**        | `rust\nif let Ok(pid) = env::var("FIREBASE_PROJECT_ID") {\n    if claims.aud != pid { return Err(...); }\n    let expected_iss = format!("https://securetoken.google.com/{}", pid);\n    if claims.iss != expected_iss { return Err(...); }\n}\n// If FIREBASE_PROJECT_ID is NOT set, aud/iss are NOT checked!\n` |
| **Fix**                 | Make `FIREBASE_PROJECT_ID` a **mandatory** configuration and use standard JWT validation: \n```rust\npub async fn verify_firebase_token(token: &str) -> Result<String, FaultReportError> {\n let certs = fetch_certs().await?;\n let header = jsonwebtoken::decode_header(token)\n .map_err(                      | \_  | FaultReportError::Unauthorized)?;\n let kid = header.kid.ok_or(FaultReportError::Unauthorized)?;\n let pem = certs.get(&kid).ok_or(Fault_REPORTError::Unauthorized)?;\n \n let decoding_key = DecodingKey::from_rsa_pem(pem.as_bytes())\n .map_err( | \_  | FaultReportError::Unauthorized)?;\n \n let project_id = env::var("FIREBASE_PROJECT_ID")\n .map_err( | \_  | FaultReportError::Unauthorized)?;\n \n let mut validation = Validation::new(Algorithm::RS256);\n validation.set_audience(&[&project_id]);\n validation.set_issuer(&[&format!("https://securetoken.google.com/{}", project_id)]);\n validation.validate_exp = true;\n validation.required_spec_claims.insert("sub".to_string());\n \n let token_data = decode::<Claims>(token, &decoding_key, &validation)\n .map_err( | \_  | FaultReportError::Unauthorized)?;\n \n Ok(token_data.claims.sub)\n}\n``` |
| **Acceptance Criteria** | - [ ] `FIREBASE_PROJECT_ID` is mandatory — app fails startup if missing \n- [ ] `aud`, `iss`, and `exp` are always validated \n- [ ] `sub` claim is required \n- [ ] Rejects tokens from other Firebase projects                                                                                                  |

---

## 🟠 High Severity Issues (FIX Before Public GitHub Release)

### HIGH-001: Overly Permissive CORS Configuration

| Field                   | Details                                                                                                                                                                                                                                                                                                                                                                                    |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Severity**            | 🟠 High                                                                                                                                                                                                                                                                                                                                                                                    |
| **File**                | `backend/src/orchestrator.rs` (lines 9–18)                                                                                                                                                                                                                                                                                                                                                 |
| **Problem**             | `.allow_any_method()`, `.allow_any_header()`, and `.send_wildcard()` are all enabled. This allows any website to make cross-origin requests with any HTTP method and any headers.                                                                                                                                                                                                          |
| **Impact**              | \n- CSRF attacks via cross-origin requests \n- Credential leakage \n- Relaxed security posture that violates principle of least privilege                                                                                                                                                                                                                                                  |
| **Current Code**        | `rust\nlet mut cors = Cors::default()\n    .allow_any_method()\n    .allow_any_header()\n    .send_wildcard();\n`                                                                                                                                                                                                                                                                          |
| **Fix**                 | Restrict to specific methods and headers needed by the frontend: \n`rust\nlet mut cors = Cors::default()\n    .allowed_methods(vec!["GET", "POST"])\n    .allowed_headers(vec![\n        actix_web::http::header::AUTHORIZATION,\n        actix_web::http::header::ACCEPT,\n        actix_web::http::header::CONTENT_TYPE,\n    ])\n    .max_age(3600);\n    // REMOVE .send_wildcard()\n` |
| **Acceptance Criteria** | - [ ] Only `GET` and `POST` methods allowed \n- [ ] Only required headers explicitly listed \n- [ ] `send_wildcard()` removed \n- [ ] `max_age` set for caching                                                                                                                                                                                                                            |

---

### HIGH-002: No HTTPS / TLS Enforcement

| Field                   | Details                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Severity**            | 🟠 High                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| **File**                | `backend/src/main.rs` (line 61), `frontend/nginx.conf`, `backend/nginx.conf`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| **Problem**             | Backend binds to `0.0.0.0:8000` with no TLS. Nginx configs have no HTTPS redirect or HSTS header. All traffic is unencrypted.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| **Impact**              | \n- All API keys, auth tokens, and error data transmitted in plaintext \n- Vulnerable to man-in-the-middle attacks \n- Session hijacking                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| **Fix**                 | Add TLS termination and security headers in nginx: \n`nginx\nserver {\n    listen 80;\n    return 301 https://$host$request_uri;\n}\n\nserver {\n    listen 443 ssl http2;\n    ssl_certificate /path/to/cert.pem;\n    ssl_certificate_key /path/to/key.pem;\n    \n    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;\n    add_header X-Content-Type-Options nosniff always;\n    add_header X-Frame-Options DENY always;\n    add_header Content-Security-Policy "default-src 'self'" always;\n    \n    location /api/ {\n        proxy_pass http://backend:8000;\n        proxy_set_header Host $host;\n        proxy_set_header X-Real-IP $remote_addr;\n        proxy_set_header X-Forwarded-Proto $scheme;\n        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n    }\n}\n` |
| **Acceptance Criteria** | - [ ] HTTP → HTTPS redirect configured \n- [ ] HSTS header present on all HTTPS responses \n- [ ] TLS 1.2+ enforced \n- [ ] `X-Forwarded-Proto` passed to backend for scheme awareness                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |

---

### HIGH-003: Missing Authorization / No RBAC

| Field                   | Details                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Severity**            | 🟠 High                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| **File**                | `backend/src/handlers.rs` (lines 152–173)                                                                                                                                                                                                                                                                                                                                                                                               |
| **Problem**             | `create_project` only validates that the user is authenticated — it does not enforce any limits or verify the user should be allowed to create projects. The `list_errors` endpoint checks API key ownership but there's no middleware enforcing this consistently.                                                                                                                                                                     |
| **Impact**              | \n- Any authenticated user can create unlimited projects \n- Potential resource exhaustion \n- No audit trail of who created what                                                                                                                                                                                                                                                                                                       |
| **Fix**                 | Add a project ownership middleware and enforce limits: \n```rust\n// Add to middleware/auth.rs or new middleware/rbac.rs\npub async fn require_project_owner(\n pool: &PgPool,\n project_id: Uuid,\n user_id: Uuid,\n) -> Result<(), FaultReportError> {\n let owner: Option<Uuid> = sqlx::query_scalar(\n "SELECT created_by_user_id FROM projects WHERE id = $1"\n )\n .bind(project_id)\n .fetch_optional(pool)\n .await\n .map_err( | \_  | FaultReportError::Unauthorized)?;\n \n match owner {\n Some(id) if id == user*id => Ok(()),\n * => Err(FaultReportError::Unauthorized),\n }\n}\n\n// Enforce project creation limits\nconst MAX_PROJECTS_PER_USER: i64 = 10;\npub async fn check_project_limit(\n pool: &PgPool,\n user_id: Uuid,\n) -> Result<(), FaultReportError> {\n let count: i64 = sqlx::query_scalar(\n "SELECT COUNT(\*) FROM projects WHERE created_by_user_id = $1"\n )\n .bind(user_id)\n .fetch_one(pool)\n .await\n .map_err( | \_  | FaultReportError::Unauthorized)?;\n \n if count >= MAX_PROJECTS_PER_USER {\n return Err(FaultReportError::Validation(\n "Maximum project limit reached".to_string()\n ));\n }\n Ok(())\n}\n``` |
| **Acceptance Criteria** | - [ ] Project creation limited per user (e.g., max 10) \n- [ ] All project-scoped endpoints verify user ownership \n- [ ] Consistent middleware applied to all protected routes                                                                                                                                                                                                                                                         |

---

## 🟡 Medium Severity Issues (Address in First Post-Deployment Sprint)

### MED-001: Information Leakage in Error Responses

| Field        | Details                                                                                                                                                                                                                                             |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Severity** | 🟡 Medium                                                                                                                                                                                                                                           |
| **File**     | `backend/src/error.rs` (lines 20–37)                                                                                                                                                                                                                |
| **Problem**  | Database errors return internal details to the client. The catch-all `_ => InternalServerError` exposes `self.to_string()` which may contain SQL errors or stack traces.                                                                            |
| **Fix**      | Return generic messages to clients; log details server-side: \n`rust\n_ => {\n    tracing::error!("Internal error: {}", self);\n    HttpResponse::InternalServerError()\n        .json(serde_json::json!({"error": "Internal server error"}))\n}\n` |

---

### MED-002: O(n) API Key Verification with Timing Side-Channel

| Field        | Details                                                                                                                                                         |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Severity** | 🟡 Medium                                                                                                                                                       |
| **File**     | `backend/src/modules/projects.rs` (lines 40–63), `backend/src/auth/cache.rs` (lines 40–61)                                                                      |
| **Problem**  | `verify_api_key()` fetches ALL non-revoked projects and hashes the incoming key against each salt sequentially. This is O(n) and vulnerable to timing analysis. |
| **Fix**      | Add a prefix-indexed lookup: store first 8 chars of the API key as `api_key_prefix` and query by prefix before hashing.                                         |

---

### MED-003: High Rate Limit with No Per-IP Throttling

| Field        | Details                                                                                                             |
| ------------ | ------------------------------------------------------------------------------------------------------------------- |
| **Severity** | 🟡 Medium                                                                                                           |
| **File**     | `backend/src/modules/storage.rs` (lines 87–107)                                                                     |
| **Problem**  | 10,000 errors/hour per project with no per-IP or per-source limiting. Attackers can flood a project's error stream. |
| **Fix**      | Lower to 1,000/hour per project. Add Redis-backed per-IP rate limiting.                                             |

---

### MED-004: Basic Content Security Policy

| Field        | Details                                                                                                                                                                                                                                                                                                                                   |
| ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Severity** | 🟡 Medium                                                                                                                                                                                                                                                                                                                                 |
| **File**     | `backend/src/middleware.rs` (line 54)                                                                                                                                                                                                                                                                                                     |
| **Problem**  | CSP is only `default-src 'self'`. Missing directives for scripts, styles, images, and API connections.                                                                                                                                                                                                                                    |
| **Fix**      | Add comprehensive CSP: \n`rust\nres.headers_mut().insert(\n    CONTENT_SECURITY_POLICY,\n    "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self' https://*.firebaseapp.com; frame-ancestors 'none'; base-uri 'self'; form-action 'self'"\n        .parse().unwrap()\n);\n` |

---

### MED-005: Missing Security Headers

| Field        | Details                                                                                                                                                                                                                                                                                                               |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Severity** | 🟡 Medium                                                                                                                                                                                                                                                                                                             |
| **File**     | `backend/src/middleware.rs`                                                                                                                                                                                                                                                                                           |
| **Problem**  | No `Referrer-Policy`, `Permissions-Policy`, or `Strict-Transport-Security` headers.                                                                                                                                                                                                                                   |
| **Fix**      | Add to middleware: \n`rust\nres.headers_mut().insert(\n    actix_web::http::header::REFERRER_POLICY,\n    "strict-origin-when-cross-origin".parse().unwrap()\n);\nres.headers_mut().insert(\n    actix_web::http::header::PERMISSIONS_POLICY,\n    "camera=(), microphone=(), geolocation=()".parse().unwrap()\n);\n` |

---

## 🟢 Positive Security Findings (Keep These)

The following security practices are already in place and should be maintained:

1. ✅ **No hardcoded secrets** — all credentials use environment variables
2. ✅ **`.gitignore` excludes secrets** — `.env`, `.env.*`, `node_modules/`, `target/` properly ignored
3. ✅ **SQL Injection Protection** — all queries use parameterized `sqlx::query(...).bind()`
4. ✅ **API Keys are Hashed + Salted** — SHA-256 with unique per-project salts
5. ✅ **Basic Security Headers Present** — CSP, X-Content-Type-Options, X-Frame-Options
6. ✅ **Source Maps Disabled** — `sourcemap: false` in Vite config
7. ✅ **Circuit Breaker on Alerts** — prevents cascading external call failures

---

## 📋 Implementation Checklist

Copy this checklist into your issue tracker (GitHub Issues / Jira / etc.):

### 🔴 Critical (Block Release)

- [ ] **CRIT-001**: Replace weak `Uuid::new_v4()` API key generation with `OsRng`-based `api_key.rs` implementation
- [ ] **CRIT-002**: Remove all auth bypass env vars and `mock:` token support from production code
- [ ] **CRIT-003**: Make `FIREBASE_PROJECT_ID` mandatory and enforce `aud`/`iss`/`exp` JWT validation

### 🟠 High (Block Public Repo)

- [ ] **HIGH-001**: Restrict CORS to `GET`/`POST` only, explicit headers, remove `send_wildcard()`
- [ ] **HIGH-002**: Add HTTPS redirect, TLS termination, and HSTS in nginx
- [ ] **HIGH-003**: Add per-user project limits and project ownership middleware

### 🟡 Medium (First Sprint After Release)

- [ ] **MED-001**: Sanitize error responses — generic messages to clients, detailed logs server-side
- [ ] **MED-002**: Optimize API key lookup with prefix indexing
- [ ] **MED-003**: Lower rate limit to ~1,000/hour and add per-IP limiting
- [ ] **MED-004**: Expand CSP with script-src, style-src, connect-src, frame-ancestors
- [ ] **MED-005**: Add Referrer-Policy and Permissions-Policy headers

### 🟢 Hardening (Ongoing)

- [ ] Run `cargo audit` and `npm audit` — address all reported vulnerabilities
- [ ] Add non-root user to backend Dockerfile
- [ ] Set `RUST_BACKTRACE=0` in production docker-compose
- [ ] Add backend Docker health check
- [ ] Consider adding frontend CSP meta tags in `index.html`

---

## 📎 Related Files for Reference

| File                              | Purpose                                   |
| --------------------------------- | ----------------------------------------- |
| `backend/src/modules/projects.rs` | API key generation & verification         |
| `backend/src/api_key.rs`          | **Secure** API key generation (use this!) |
| `backend/src/middleware/auth.rs`  | Auth middleware with bypasses             |
| `backend/src/auth/firebase.rs`    | Firebase JWT verification                 |
| `backend/src/orchestrator.rs`     | CORS configuration                        |
| `backend/src/handlers.rs`         | HTTP handlers                             |
| `backend/src/error.rs`            | Error response formatting                 |
| `backend/src/middleware.rs`       | Security headers middleware               |
| `frontend/nginx.conf`             | Reverse proxy / TLS config                |
| `docker-compose.yml`              | Production environment variables          |

---

## 🚨 Do Not Deploy Until

All 🔴 **Critical** and 🟠 **High** items above are resolved, tested, and verified.
