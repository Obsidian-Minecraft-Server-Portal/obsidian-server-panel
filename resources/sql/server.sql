-- This SQL script creates a table for managing Minecraft servers in a database.
-- It includes fields for server configuration, status, and ownership.
-- Ensure you have a `users` table created before running this script, as it references the `users` table for the owner_id foreign key.

CREATE TABLE IF NOT EXISTS `servers`
(
	`id` INT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,         -- unique identifier for the server
	`name`              VARCHAR(255) NOT NULL,                           -- e.g. 'My Minecraft Server'
	`directory`         VARCHAR(255) NOT NULL,                           -- the directory name where the server files are stored
	`java_executable`   VARCHAR(512) NOT NULL,                           -- path to the Java executable
	`java_args`         TEXT NOT NULL,                                   -- additional arguments for Java (no default for TEXT in MySQL)
	`max_memory`        TINYINT NOT NULL DEFAULT 4,                      -- in GB
	`min_memory`        TINYINT NOT NULL DEFAULT 2,                      -- in GB
	`minecraft_args`    VARCHAR(255) NOT NULL DEFAULT 'nogui',
	`server_jar`        VARCHAR(255) NOT NULL DEFAULT '',
	`upnp`              TINYINT NOT NULL DEFAULT 0,                      -- 0 = false, 1 = true
	`status`            TINYINT NOT NULL DEFAULT 0,                      -- 0 => idle, 1 => running, etc.
	`auto_start`        BOOLEAN NOT NULL DEFAULT 0,
	`auto_restart`      BOOLEAN NOT NULL DEFAULT 1,
	`backup_enabled`    BOOLEAN NOT NULL DEFAULT 1,
	`backup_cron`       VARCHAR(50) NOT NULL DEFAULT '0 0 * * * *',
	`backup_retention`  INT NOT NULL DEFAULT 7,
	`description`       TEXT,                                            -- a short description (nullable, no default)
	`minecraft_version` VARCHAR(50) DEFAULT '',
	`server_type`       TINYINT NOT NULL DEFAULT 0,
	`loader_version`    VARCHAR(50) DEFAULT NULL,
	`owner_id` INT UNSIGNED NOT NULL,
	`created_at`        INT NOT NULL DEFAULT (UNIX_TIMESTAMP()),         -- timestamp in seconds since epoch
	`updated_at`        INT NOT NULL DEFAULT (UNIX_TIMESTAMP()),         -- timestamp in seconds since epoch
	`last_started`      INT DEFAULT NULL,
	`last_update_check` INT DEFAULT NULL,
	`update_available`  BOOLEAN NOT NULL DEFAULT 0,
	`latest_version`    VARCHAR(50) DEFAULT NULL,
	FOREIGN KEY (`owner_id`) REFERENCES `users` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
);
