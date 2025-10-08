CREATE TABLE IF NOT EXISTS java_version_map
(
    java_version TEXT PRIMARY KEY NOT NULL,
    min_version  TEXT             NOT NULL,
    max_version  TEXT             NOT NULL,
    updated_at   TEXT             NOT NULL DEFAULT (DATETIME('now'))
);
