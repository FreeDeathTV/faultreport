# Issue: Replace Firebase auth stub with configurable auth client

**Priority:** Medium

**Summary:** Replace `frontend/src/auth/firebase.ts` stub with a configurable auth client that can be switched between a stub and real Firebase via env or build flag.

**Acceptance criteria:**

- `AuthClient` interface implemented with `StubAuthClient` and `FirebaseAuthClient`
- Frontend reads `VITE_USE_STUB_AUTH` (or similar) to toggle behavior
- E2E test coverage for login flow using test credentials or stubbed behavior

**Implementation steps:**

1. Define `AuthClient` interface in `frontend/src/auth/types.ts`.
2. Implement `StubAuthClient` and `FirebaseAuthClient` and wire in `main.tsx` via env flag.
3. Add simple E2E test using Playwright or Cypress that runs against stubbed auth.

**Estimated effort:** 4-8 hours
