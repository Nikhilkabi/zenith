CREATE TABLE IF NOT EXISTS trips (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    when_text TEXT,
    year INTEGER,
    currency TEXT,
    notes TEXT,
    status TEXT NOT NULL DEFAULT 'dream' CHECK(status IN ('dream', 'done')),
    cover_relpath TEXT,
    cover_x REAL NOT NULL DEFAULT 50,
    cover_y REAL NOT NULL DEFAULT 50,
    sort INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    deleted_at TEXT
);

CREATE TABLE IF NOT EXISTS trip_stops (
    trip_id TEXT NOT NULL,
    place_id TEXT NOT NULL,
    sort INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (trip_id, place_id)
);

CREATE TABLE IF NOT EXISTS trip_costs (
    trip_id TEXT NOT NULL,
    category TEXT NOT NULL,
    amount INTEGER NOT NULL,
    PRIMARY KEY (trip_id, category)
);

CREATE TABLE IF NOT EXISTS place_tasks (
    id TEXT PRIMARY KEY,
    place_id TEXT NOT NULL,
    body TEXT NOT NULL,
    done INTEGER NOT NULL DEFAULT 0,
    sort INTEGER NOT NULL DEFAULT 0
);

ALTER TABLE places ADD COLUMN when_text TEXT;
ALTER TABLE places ADD COLUMN year INTEGER;
