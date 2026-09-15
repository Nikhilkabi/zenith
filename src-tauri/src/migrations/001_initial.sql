CREATE TABLE IF NOT EXISTS countries (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    iso TEXT,
    cover_relpath TEXT,
    sort INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS places (
    id TEXT PRIMARY KEY,
    country_id TEXT NOT NULL REFERENCES countries(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    cover_relpath TEXT,
    notes TEXT,
    status TEXT NOT NULL CHECK(status IN ('dream', 'been')),
    sort INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS images (
    id TEXT PRIMARY KEY,
    place_id TEXT NOT NULL REFERENCES places(id) ON DELETE CASCADE,
    original_relpath TEXT NOT NULL,
    display_relpath TEXT NOT NULL,
    thumb_relpath TEXT NOT NULL,
    sort INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS links (
    id TEXT PRIMARY KEY,
    place_id TEXT REFERENCES places(id) ON DELETE SET NULL,
    country_id TEXT REFERENCES countries(id) ON DELETE SET NULL,
    url TEXT NOT NULL,
    title TEXT,
    thumb_relpath TEXT,
    source TEXT NOT NULL CHECK(source IN ('youtube', 'web'))
);

CREATE TABLE IF NOT EXISTS inbox_items (
    id TEXT PRIMARY KEY,
    url TEXT NOT NULL,
    title TEXT,
    thumb_relpath TEXT,
    image_relpath TEXT,
    source TEXT NOT NULL,
    created_at TEXT NOT NULL,
    filed_at TEXT
);
