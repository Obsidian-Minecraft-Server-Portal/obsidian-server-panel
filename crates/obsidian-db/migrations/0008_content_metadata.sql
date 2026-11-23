ALTER TABLE mods ADD COLUMN mod_id TEXT;
ALTER TABLE mods ADD COLUMN depends TEXT NOT NULL DEFAULT '[]';
ALTER TABLE mods ADD COLUMN metadata_version INTEGER NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS idx_mods_mod_id ON mods (instance_id, mod_id);
