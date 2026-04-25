pub const SCHEMA: &str = r#"
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS manga (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    file_path TEXT NOT NULL UNIQUE,
    file_hash TEXT NOT NULL,
    folder TEXT NOT NULL,
    page_count INTEGER NOT NULL DEFAULT 0,
    score REAL NOT NULL DEFAULT 0,
    cover_cache_path TEXT,
    eh_gid TEXT,
    eh_token TEXT,
    eh_title TEXT,
    eh_title_jpn TEXT,
    eh_category TEXT,
    eh_posted TEXT,
    eh_uploader TEXT,
    eh_filesize INTEGER,
    eh_thumb_url TEXT,
    tag_status TEXT NOT NULL DEFAULT 'non-tag',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    namespace TEXT NOT NULL,
    tag TEXT NOT NULL,
    UNIQUE(namespace, tag)
);

CREATE TABLE IF NOT EXISTS manga_tags (
    manga_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (manga_id, tag_id),
    FOREIGN KEY (manga_id) REFERENCES manga(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS read_progress (
    manga_id INTEGER PRIMARY KEY,
    current_page INTEGER NOT NULL DEFAULT 0,
    last_read_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (manga_id) REFERENCES manga(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_manga_folder ON manga(folder);
CREATE INDEX IF NOT EXISTS idx_manga_tag_status ON manga(tag_status);
CREATE INDEX IF NOT EXISTS idx_manga_score ON manga(score);
CREATE INDEX IF NOT EXISTS idx_tags_namespace ON tags(namespace);
"#;