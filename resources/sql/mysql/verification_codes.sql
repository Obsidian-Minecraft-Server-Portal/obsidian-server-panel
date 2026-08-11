CREATE TABLE IF NOT EXISTS `verification_codes`
(
	`user_id`    BIGINT NOT NULL,
	`purpose`    VARCHAR(16) NOT NULL,
	`code_hash`  VARCHAR(72) NOT NULL,
	`attempts`   INT NOT NULL DEFAULT 0,
	`created_at` DATETIME NOT NULL,
	`expires_at` DATETIME NOT NULL,
	PRIMARY KEY (`user_id`, `purpose`)
);
