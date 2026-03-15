CREATE TABLE IF NOT EXISTS prompts (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
