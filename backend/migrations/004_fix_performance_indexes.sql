-- Fix for CRITICAL Severity: Full table scan on API key validation
-- Optimized lookup for ApiKeyCache misses
CREATE INDEX IF NOT EXISTS idx_projects_api_key_hash ON projects(api_key_hash);

-- Fix for CRITICAL Severity: Performance collapse on Rate Limiting
-- Optimized COUNT(*) for the rolling hour window
CREATE INDEX IF NOT EXISTS idx_errors_rate_limit_lookup ON errors(project_id, created_at);