-- 0001 — draft schema. Foreign keys are mandatory here (decision 8):
-- orphaned rows are the exact corruption class SQLite was chosen to prevent.

CREATE TABLE source (
    id   INTEGER PRIMARY KEY,
    key  TEXT NOT NULL UNIQUE,   -- adapter key, e.g. 'mangadex'
    name TEXT NOT NULL
);

CREATE TABLE user (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE,
    token_hash TEXT NOT NULL UNIQUE
);

CREATE TABLE manga (
    id         INTEGER PRIMARY KEY,
    source_id  INTEGER NOT NULL REFERENCES source(id) ON DELETE RESTRICT,
    source_key TEXT NOT NULL UNIQUE,
    title      TEXT NOT NULL
);

CREATE TABLE category (
    id               INTEGER PRIMARY KEY,
    name             TEXT NOT NULL UNIQUE,
    delete_after_read INTEGER NOT NULL DEFAULT 0 CHECK (delete_after_read IN (0, 1))
);

CREATE TABLE manga_category (
    manga_id    INTEGER NOT NULL REFERENCES manga(id) ON DELETE CASCADE,
    category_id INTEGER NOT NULL REFERENCES category(id) ON DELETE CASCADE,
    PRIMARY KEY (manga_id, category_id)
);

CREATE TABLE chapter (
    id         INTEGER PRIMARY KEY,
    manga_id   INTEGER NOT NULL REFERENCES manga(id) ON DELETE CASCADE,
    source_key TEXT NOT NULL,
    number     TEXT NOT NULL,
    downloaded INTEGER NOT NULL DEFAULT 0 CHECK (downloaded IN (0, 1)),
    UNIQUE (manga_id, source_key)
);

CREATE TABLE reading_progress (
    id           INTEGER PRIMARY KEY,
    user_id      INTEGER NOT NULL REFERENCES user(id) ON DELETE CASCADE,
    chapter_id   INTEGER NOT NULL REFERENCES chapter(id) ON DELETE CASCADE,
    page_n       INTEGER NOT NULL CHECK (page_n >= 0),
    scroll_ratio REAL    NOT NULL CHECK (scroll_ratio >= 0.0 AND scroll_ratio <= 1.0),
    UNIQUE (user_id, chapter_id)
);
