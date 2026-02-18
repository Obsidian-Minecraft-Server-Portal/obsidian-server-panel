CREATE TABLE IF NOT EXISTS installed_mods
(
	id            INTEGER PRIMARY KEY AUTOINCREMENT,
	mod_id        TEXT NOT NULL,
	name          TEXT NOT NULL,
	version       TEXT NOT NULL,
	author        TEXT NOT NULL,
	description   TEXT NOT NULL,
	icon          TEXT,
	modrinth_id   TEXT,
	curseforge_id TEXT,
	filename      TEXT,
	server_id     INTEGER NOT NULL REFERENCES servers(id) ON DELETE CASCADE
);
