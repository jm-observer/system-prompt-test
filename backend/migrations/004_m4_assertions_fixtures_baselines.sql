CREATE TABLE assertions (
    id TEXT PRIMARY KEY NOT NULL,
    test_case_id TEXT NOT NULL REFERENCES test_cases(id) ON DELETE CASCADE,
    assertion_type TEXT NOT NULL CHECK(assertion_type IN ('must_call', 'must_not_call', 'whitelist', 'keyword_present', 'keyword_absent')),
    config TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
);

CREATE TABLE assertion_results (
    id TEXT PRIMARY KEY NOT NULL,
    run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    assertion_id TEXT NOT NULL REFERENCES assertions(id) ON DELETE CASCADE,
    passed INTEGER NOT NULL, -- 1 for true, 0 for false
    evidence TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE fixtures (
    id TEXT PRIMARY KEY NOT NULL,
    run_id TEXT NOT NULL REFERENCES runs(id),
    request_snapshot TEXT NOT NULL, -- JSON
    response_snapshot TEXT NOT NULL, -- JSON
    created_at TEXT NOT NULL
);

CREATE TABLE baselines (
    id TEXT PRIMARY KEY NOT NULL,
    test_case_id TEXT NOT NULL REFERENCES test_cases(id) ON DELETE CASCADE,
    run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    marked_at TEXT NOT NULL,
    UNIQUE(test_case_id) -- One baseline per test case
);
