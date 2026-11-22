CREATE TABLE IF NOT EXISTS serverpack_decisions (
	sha1 TEXT PRIMARY KEY NOT NULL,
	include INTEGER NOT NULL,
	updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);
