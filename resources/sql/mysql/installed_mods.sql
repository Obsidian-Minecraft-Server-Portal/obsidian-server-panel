CREATE TABLE IF NOT EXISTS `installed_mods`
(
	`id`            INT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
	`mod_id`        TEXT NOT NULL,
	`name`          TEXT NOT NULL,
	`version`       TEXT NOT NULL,
	`author`        TEXT NOT NULL,
	`description`   TEXT NOT NULL,
	`icon`          TEXT,
	`modrinth_id`   TEXT,
	`curseforge_id` TEXT,
	`filename`      TEXT,
	`server_id`     INT UNSIGNED NOT NULL,
	FOREIGN KEY (`server_id`) REFERENCES `servers` (`id`) ON DELETE CASCADE
);
