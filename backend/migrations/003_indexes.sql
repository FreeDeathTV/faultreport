-- Migration 003: Additional Indexes for Performance

-- API key validation lookup
CREATE INDEX IF NOT EXISTS idx_projects_api_key_hash ON projects(api_key_hash);

-- Error deduplication lookup
CREATE UNIQUE INDEX IF NOT EXISTS idx_errors_project_hash ON errors(project_id, hash);

-- Error listing sort order
CREATE INDEX IF NOT EXISTS idx_errors_project_created ON errors(project_id, last_seen_at DESC);

-- Rate limit window lookups
CREATE INDEX IF NOT EXISTS idx_rate_limit_window ON rate_limit_tracker(hour_window);

-- Alert dedup lookups (can't use volatile functions in predicates)
CREATE INDEX IF NOT EXISTS idx_alert_dedup_recent ON alert_dedup(project_id, error_hash, last_alert_at);

