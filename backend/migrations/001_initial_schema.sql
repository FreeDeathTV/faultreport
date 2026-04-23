-- Migration 001: Initial Schema (FaultReport MVP)

-- Users (Firebase auth)
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  firebase_uid VARCHAR(255) UNIQUE NOT NULL,
  email VARCHAR(255) NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Projects (per-user, API key per project)
CREATE TABLE projects (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  created_by_user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
  name VARCHAR(255) NOT NULL,
  api_key_hash VARCHAR(64) NOT NULL,
  api_key_salt VARCHAR(32) NOT NULL,
  revoked_at TIMESTAMP,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_projects_api_key_hash ON projects(api_key_hash);

-- Errors (deterministic grouping)
CREATE TABLE errors (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
  hash VARCHAR(64) NOT NULL,
  message TEXT NOT NULL,
  stack TEXT,
  context JSONB,
  count BIGINT DEFAULT 1,
  first_seen_at TIMESTAMP NOT NULL DEFAULT NOW(),
  last_seen_at TIMESTAMP NOT NULL DEFAULT NOW(),
  status VARCHAR(20) DEFAULT 'active',
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_errors_project_hash ON errors(project_id, hash);
CREATE INDEX idx_errors_project_status ON errors(project_id, status);
CREATE INDEX idx_errors_last_seen ON errors(last_seen_at DESC);

-- Ledger (append-only audit)
CREATE TABLE ledger (
  id BIGSERIAL PRIMARY KEY,
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
  error_id UUID NOT NULL REFERENCES errors(id) ON DELETE RESTRICT,
  event_type VARCHAR(50) NOT NULL, -- 'new_error', 'duplicate'
  data JSONB,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ledger_project ON ledger(project_id);

-- Rate limits (10K/hour per project)
CREATE TABLE rate_limit_tracker (
  project_id UUID PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
  hour_window TIMESTAMP NOT NULL,
  error_count BIGINT DEFAULT 0,
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Spike alert dedup (5min window)
CREATE TABLE alert_dedup (
  id BIGSERIAL PRIMARY KEY,
  project_id UUID NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
  error_hash VARCHAR(64) NOT NULL,
  last_alert_at TIMESTAMP NOT NULL DEFAULT NOW(),
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_alert_dedup_project_hash ON alert_dedup(project_id, error_hash);

