CREATE TABLE users (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    username       TEXT    NOT NULL UNIQUE,
    email          TEXT    DEFAULT NULL,
    password_hash  TEXT    NOT NULL,
    is_admin       INTEGER NOT NULL DEFAULT 0,
    instance_limit INTEGER NOT NULL DEFAULT 0,
    totp_secret    TEXT    DEFAULT NULL,
    totp_enabled   INTEGER NOT NULL DEFAULT 0,
    created_at     INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE user_recovery_codes (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id   INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    code_hash TEXT    NOT NULL,
    used_at   INTEGER DEFAULT NULL
);
CREATE INDEX idx_recovery_user ON user_recovery_codes (user_id);

CREATE TABLE user_trusted_ips (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    ip         TEXT    NOT NULL,
    first_seen INTEGER NOT NULL DEFAULT (unixepoch()),
    last_seen  INTEGER NOT NULL DEFAULT (unixepoch()),
    user_agent TEXT    DEFAULT NULL,
    UNIQUE (user_id, ip)
);

CREATE TABLE sessions (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id      INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    token_hash   TEXT    NOT NULL UNIQUE,
    ip           TEXT    DEFAULT NULL,
    user_agent   TEXT    DEFAULT NULL,
    created_at   INTEGER NOT NULL DEFAULT (unixepoch()),
    last_seen_at INTEGER NOT NULL DEFAULT (unixepoch()),
    expires_at   INTEGER NOT NULL
);
CREATE INDEX idx_sessions_user ON sessions (user_id);

CREATE TABLE api_tokens (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id      INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name         TEXT    NOT NULL,
    token_hash   TEXT    NOT NULL UNIQUE,
    created_at   INTEGER NOT NULL DEFAULT (unixepoch()),
    expires_at   INTEGER DEFAULT NULL,
    last_used_at INTEGER DEFAULT NULL
);
CREATE INDEX idx_api_tokens_user ON api_tokens (user_id);

CREATE TABLE invites (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    token_hash TEXT    NOT NULL UNIQUE,
    created_by INTEGER DEFAULT NULL REFERENCES users (id) ON DELETE SET NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    expires_at INTEGER NOT NULL,
    used_at    INTEGER DEFAULT NULL,
    used_by    INTEGER DEFAULT NULL REFERENCES users (id) ON DELETE SET NULL
);

CREATE TABLE password_resets (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    token_hash TEXT    NOT NULL UNIQUE,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    expires_at INTEGER NOT NULL,
    used_at    INTEGER DEFAULT NULL
);

CREATE TABLE email_verification_codes (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    code_hash  TEXT    NOT NULL,
    ip         TEXT    DEFAULT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    expires_at INTEGER NOT NULL,
    used_at    INTEGER DEFAULT NULL
);

CREATE TABLE security_events (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER DEFAULT NULL REFERENCES users (id) ON DELETE SET NULL,
    kind       TEXT    NOT NULL,
    ip         TEXT    DEFAULT NULL,
    details    TEXT    DEFAULT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX idx_security_events_user ON security_events (user_id);

CREATE TABLE instances (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    owner_id         INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name             TEXT    NOT NULL,
    loader           TEXT    NOT NULL,
    mc_version       TEXT    NOT NULL,
    loader_version   TEXT    DEFAULT NULL,
    java_runtime_id INTEGER DEFAULT NULL REFERENCES java_runtimes (id) ON DELETE SET NULL,
    jvm_args         TEXT    NOT NULL DEFAULT '',
    memory_min      INTEGER NOT NULL DEFAULT 1024,
    memory_max      INTEGER NOT NULL DEFAULT 2048,
    port            INTEGER NOT NULL DEFAULT 25565,
    upnp_enabled    INTEGER NOT NULL DEFAULT 0,
    auto_start      INTEGER NOT NULL DEFAULT 0,
    icon_path       TEXT    DEFAULT NULL,
    directory       TEXT    NOT NULL,
    created_at      INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at      INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX idx_instances_owner ON instances (owner_id);

CREATE TABLE instance_members (
    instance_id INTEGER NOT NULL REFERENCES instances (id) ON DELETE CASCADE,
    user_id     INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    role        TEXT    NOT NULL,
    permissions INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (instance_id, user_id)
);
CREATE INDEX idx_instance_members_user ON instance_members (user_id);

CREATE TABLE minecraft_versions (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    loader         TEXT    NOT NULL,
    mc_version     TEXT    NOT NULL,
    loader_version TEXT    NOT NULL DEFAULT '',
    jar_path       TEXT    NOT NULL,
    installed_at   INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE (loader, mc_version, loader_version)
);

CREATE TABLE java_runtimes (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    source       TEXT    NOT NULL CHECK (source IN ('piston', 'system')),
    component    TEXT    DEFAULT NULL,
    version      TEXT    NOT NULL,
    path         TEXT    NOT NULL UNIQUE,
    arch         TEXT    NOT NULL,
    installed_at INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE mods
(
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    filename    TEXT    NOT NULL,
    project_id  TEXT    DEFAULT NULL,
    version_id  TEXT    DEFAULT NULL,
    icon        BLOB    DEFAULT NULL,
    platform    INTEGER DEFAULT NULL,
    kind        INTEGER NOT NULL DEFAULT 0,
    instance_id INTEGER NOT NULL REFERENCES instances (id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_mods_instance ON mods (instance_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_mods_instance_file ON mods (instance_id, filename);

CREATE TABLE backup_policies (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    instance_id  INTEGER NOT NULL REFERENCES instances (id) ON DELETE CASCADE,
    keep_hourly  INTEGER NOT NULL DEFAULT 0,
    keep_daily   INTEGER NOT NULL DEFAULT 7,
    keep_weekly  INTEGER NOT NULL DEFAULT 4,
    keep_monthly INTEGER NOT NULL DEFAULT 3,
    excludes     TEXT    NOT NULL DEFAULT '[]',
    created_at   INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE backup_snapshots (
    id           TEXT    PRIMARY KEY,
    instance_id  INTEGER NOT NULL REFERENCES instances (id) ON DELETE CASCADE,
    policy_id    INTEGER DEFAULT NULL REFERENCES backup_policies (id) ON DELETE SET NULL,
    label        TEXT    NOT NULL DEFAULT '',
    file_count   INTEGER NOT NULL DEFAULT 0,
    size_bytes   INTEGER NOT NULL DEFAULT 0,
    created_at   INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX idx_backup_snapshots_instance ON backup_snapshots (instance_id);

CREATE TABLE schedules (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    instance_id   INTEGER NOT NULL REFERENCES instances (id) ON DELETE CASCADE,
    name          TEXT    NOT NULL,
    enabled       INTEGER NOT NULL DEFAULT 1,
    schedule_json TEXT    NOT NULL,
    actions_json  TEXT    NOT NULL,
    next_run_at   INTEGER DEFAULT NULL,
    created_at    INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at    INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX idx_schedules_instance ON schedules (instance_id);

CREATE TABLE schedule_runs (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    schedule_id INTEGER NOT NULL REFERENCES schedules (id) ON DELETE CASCADE,
    started_at  INTEGER NOT NULL DEFAULT (unixepoch()),
    finished_at INTEGER DEFAULT NULL,
    status      TEXT    NOT NULL,
    message     TEXT    NOT NULL DEFAULT ''
);
CREATE INDEX idx_schedule_runs_schedule ON schedule_runs (schedule_id);

CREATE TABLE notifications (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id    INTEGER DEFAULT NULL REFERENCES users (id) ON DELETE CASCADE,
    kind       TEXT    NOT NULL,
    title      TEXT    NOT NULL,
    body       TEXT    NOT NULL DEFAULT '',
    read_at    INTEGER DEFAULT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX idx_notifications_user ON notifications (user_id);

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
