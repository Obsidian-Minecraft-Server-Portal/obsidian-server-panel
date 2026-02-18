CREATE TABLE IF NOT EXISTS users
(
	id                    INTEGER PRIMARY KEY AUTOINCREMENT,
	username              TEXT NOT NULL UNIQUE,
	password              TEXT NOT NULL,
	permissions           INTEGER NOT NULL DEFAULT 0,
	join_date             TEXT NOT NULL DEFAULT (DATETIME('now')),
	last_online           TEXT NOT NULL DEFAULT (DATETIME('now')),
	needs_password_change INTEGER NOT NULL DEFAULT 0,
	is_active             INTEGER NOT NULL DEFAULT 1
);
