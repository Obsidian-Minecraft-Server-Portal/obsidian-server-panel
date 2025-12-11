-- This SQL script creates tables for managing server backups in a database.
-- Backup schedules for automated backups are stored in the database.
-- Actual backup metadata is managed by the obsidian-backups crate using git.

CREATE TABLE IF NOT EXISTS `backup_schedules`
(
	`id` INT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,         -- unique identifier for the schedule
	`server_id` INT UNSIGNED NOT NULL,                                    -- reference to the server
	`interval_amount` INT NOT NULL,                                    -- interval amount (e.g., 6)
	`interval_unit`   VARCHAR(20) NOT NULL,                            -- 'hours', 'days', or 'weeks'
	`backup_type`     TINYINT NOT NULL DEFAULT 0,                      -- 0 => full, 1 => incremental, 2 => world
	`enabled`         BOOLEAN NOT NULL DEFAULT 1,
	`retention_days`  INT DEFAULT 7,
	`last_run`        INT DEFAULT NULL,
	`next_run`        INT DEFAULT NULL,
	`created_at`      INT NOT NULL DEFAULT (UNIX_TIMESTAMP()),         -- timestamp in seconds
	`updated_at`      INT NOT NULL DEFAULT (UNIX_TIMESTAMP()),         -- timestamp in seconds
	FOREIGN KEY (`server_id`) REFERENCES `servers` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
);
