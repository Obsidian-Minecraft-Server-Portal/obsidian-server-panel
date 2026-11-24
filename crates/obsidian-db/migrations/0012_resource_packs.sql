CREATE TABLE IF NOT EXISTS resource_packs
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    instance_id INTEGER NOT NULL REFERENCES instances (id) ON DELETE CASCADE,
    position    INTEGER NOT NULL,
    name        TEXT    NOT NULL,
    filename    TEXT    NOT NULL,
    platform    INTEGER DEFAULT NULL,
    project_id  TEXT    DEFAULT NULL,
    version_id  TEXT    DEFAULT NULL,
    size        INTEGER NOT NULL,
    sha1        TEXT    NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_resource_packs_instance ON resource_packs (instance_id, position);

CREATE TABLE IF NOT EXISTS resource_pack_state
(
    instance_id INTEGER PRIMARY KEY REFERENCES instances (id) ON DELETE CASCADE,
    token       TEXT    NOT NULL,
    origin      TEXT    DEFAULT NULL,
    url         TEXT    DEFAULT NULL,
    sha1        TEXT    DEFAULT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_resource_pack_state_token ON resource_pack_state (token);
