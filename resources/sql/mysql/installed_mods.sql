CREATE TABLE IF NOT EXISTS `installed_mods`
(
	`id`            BIGINT PRIMARY KEY AUTO_INCREMENT,
	`mod_id`        TEXT NOT NULL,
	`name`          TEXT NOT NULL,
	`version`       TEXT NOT NULL,
	`author`        TEXT NOT NULL,
	`description`   TEXT NOT NULL,
	`icon`          MEDIUMTEXT,
	`modrinth_id`   TEXT,
	`curseforge_id` TEXT,
	`filename`      VARCHAR(512),
	`server_id`     BIGINT NOT NULL,
	FOREIGN KEY (`server_id`) REFERENCES `servers` (`id`) ON DELETE CASCADE,
	INDEX `idx_installed_mods_server_id` (`server_id`)
);
