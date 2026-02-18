CREATE TABLE IF NOT EXISTS `backup_schedules`
(
	`id` INT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
	`server_id` INT UNSIGNED NOT NULL,
	`interval_amount` INT NOT NULL,
	`interval_unit`   VARCHAR(20) NOT NULL,
	`backup_type`     TINYINT NOT NULL DEFAULT 0,
	`enabled`         BOOLEAN NOT NULL DEFAULT 1,
	`retention_days`  INT DEFAULT 7,
	`last_run`        INT DEFAULT NULL,
	`next_run`        INT DEFAULT NULL,
	`created_at`      INT NOT NULL DEFAULT (UNIX_TIMESTAMP()),
	`updated_at`      INT NOT NULL DEFAULT (UNIX_TIMESTAMP()),
	FOREIGN KEY (`server_id`) REFERENCES `servers` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
);
