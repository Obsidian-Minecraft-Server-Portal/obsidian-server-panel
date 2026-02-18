CREATE TABLE IF NOT EXISTS backup_schedules
(
	id              SERIAL PRIMARY KEY,
	server_id       INT NOT NULL REFERENCES servers(id) ON DELETE CASCADE ON UPDATE CASCADE,
	interval_amount INT NOT NULL,
	interval_unit   VARCHAR(20) NOT NULL,
	backup_type     SMALLINT NOT NULL DEFAULT 0,
	enabled         BOOLEAN NOT NULL DEFAULT TRUE,
	retention_days  INT DEFAULT 7,
	last_run        BIGINT DEFAULT NULL,
	next_run        BIGINT DEFAULT NULL,
	created_at      BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT),
	updated_at      BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW())::BIGINT)
);
