CREATE TABLE IF NOT EXISTS instance_sources
(
    instance_id INTEGER NOT NULL REFERENCES instances (id) ON DELETE CASCADE,
    slot        TEXT    NOT NULL,
    platform    INTEGER NOT NULL,
    project_id  TEXT    NOT NULL,
    version_id  TEXT    NOT NULL,
    target      TEXT    NOT NULL,
    PRIMARY KEY (instance_id, slot)
);
