-- Migration v6: Reports, Audit Logs and Pricing
-- M6: Report System, CI Integration, and Security Hardening

-- Run Reports table
CREATE TABLE IF NOT EXISTS run_reports (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    total_latency_ms INTEGER DEFAULT 0,
    total_tokens INTEGER DEFAULT 0,
    estimated_cost_usd REAL DEFAULT 0,
    assertion_passed_count INTEGER DEFAULT 0,
    assertion_failed_count INTEGER DEFAULT 0,
    failure_reason TEXT, -- 'assertion', 'timeout', 'api_error', 'model_refusal'
    created_at TEXT NOT NULL,
    FOREIGN KEY (run_id) REFERENCES runs(id) ON DELETE CASCADE
);

-- Model Pricing table
CREATE TABLE IF NOT EXISTS model_pricing (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    input_1k_tokens_usd REAL NOT NULL,
    output_1k_tokens_usd REAL NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (model_id) REFERENCES models(id) ON DELETE CASCADE
);

-- Audit Logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    user_id TEXT, -- placeholder for future auth
    action TEXT NOT NULL, -- 'create', 'update', 'delete', 'run'
    resource_type TEXT NOT NULL, -- 'prompt', 'project', 'provider', etc.
    resource_id TEXT,
    metadata TEXT, -- JSON string
    created_at TEXT NOT NULL
);

-- Indicies for performance
CREATE INDEX IF NOT EXISTS idx_run_reports_run_id ON run_reports(run_id);
CREATE INDEX IF NOT EXISTS idx_model_pricing_model_id ON model_pricing(model_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at);
