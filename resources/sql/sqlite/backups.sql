CREATE TABLE IF NOT EXISTS backup_schedules
(
	id              INTEGER PRIMARY KEY AUTOINCREMENT,
	server_id       INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
	interval_amount INTEGER NOT NULL,
	interval_unit   TEXT NOT NULL,
	backup_type     INTEGER NOT NULL DEFAULT 0,
	enabled         INTEGER NOT NULL DEFAULT 1,
	retention_days  INTEGER DEFAULT 7,
	last_run        INTEGER DEFAULT NULL,
	next_run        INTEGER DEFAULT NULL,
	created_at      INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now')),
	updated_at      INTEGER NOT NULL DEFAULT (STRFTIME('%s', 'now'))
);
