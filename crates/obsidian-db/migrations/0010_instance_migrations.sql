CREATE TABLE IF NOT EXISTS instance_migrations
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    instance_id INTEGER NOT NULL REFERENCES instances (id) ON DELETE CASCADE,
    status      TEXT    NOT NULL,
    snapshot_id TEXT,
    target      TEXT    NOT NULL,
    previous    TEXT    NOT NULL,
    report      TEXT,
    error       TEXT,
    created_at  INTEGER NOT NULL DEFAULT (unixepoch()),
    finished_at INTEGER
);
CREATE INDEX IF NOT EXISTS idx_instance_migrations_instance ON instance_migrations (instance_id, id);
