CREATE TABLE IF NOT EXISTS verification_codes
(
	user_id    INTEGER NOT NULL,
	purpose    TEXT NOT NULL,
	code_hash  TEXT NOT NULL,
	attempts   INTEGER NOT NULL DEFAULT 0,
	created_at TEXT NOT NULL,
	expires_at TEXT NOT NULL,
	PRIMARY KEY (user_id, purpose)
);
