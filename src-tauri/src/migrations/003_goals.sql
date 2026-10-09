CREATE TABLE IF NOT EXISTS goals (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    notes TEXT,
    status TEXT NOT NULL DEFAULT 'dream' CHECK(status IN ('dream', 'done')),
    cover_relpath TEXT,
    sort INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS cover_credits (
    relpath TEXT PRIMARY KEY,
    credit TEXT NOT NULL,
    credit_url TEXT
);
