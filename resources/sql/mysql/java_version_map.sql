CREATE TABLE IF NOT EXISTS `java_version_map`
(
	`java_version` VARCHAR(50) PRIMARY KEY NOT NULL,
	`min_version`  VARCHAR(50) NOT NULL,
	`max_version`  VARCHAR(50) NOT NULL,
	`updated_at`   DATETIME NOT NULL DEFAULT NOW()
);
