CREATE TABLE IF NOT EXISTS instance_worldmap
(
    instance_id         INTEGER PRIMARY KEY REFERENCES instances (id) ON DELETE CASCADE,
    enabled             INTEGER NOT NULL DEFAULT 0,
    consent             INTEGER NOT NULL DEFAULT 0,
    watch_while_running INTEGER NOT NULL DEFAULT 1,
    render_after_stop   INTEGER NOT NULL DEFAULT 1,
    full_pass_hours     INTEGER NOT NULL DEFAULT 24
);
