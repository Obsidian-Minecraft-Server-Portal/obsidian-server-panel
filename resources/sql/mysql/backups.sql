CREATE TABLE IF NOT EXISTS `backup_schedules`
(
	`id` BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
	`server_id` BIGINT NOT NULL,
	`interval_amount` BIGINT NOT NULL,
	`interval_unit`   VARCHAR(20) NOT NULL,
	`backup_type`     TINYINT NOT NULL DEFAULT 0,
	`enabled`         BOOLEAN NOT NULL DEFAULT 1,
	`retention_days`  BIGINT DEFAULT 7,
	`last_run`        BIGINT DEFAULT NULL,
	`next_run`        BIGINT DEFAULT NULL,
	`created_at`      BIGINT NOT NULL DEFAULT (UNIX_TIMESTAMP()),
	`updated_at`      BIGINT NOT NULL DEFAULT (UNIX_TIMESTAMP()),
	FOREIGN KEY (`server_id`) REFERENCES `servers` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
);
