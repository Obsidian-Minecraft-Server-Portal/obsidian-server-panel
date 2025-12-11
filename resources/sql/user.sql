CREATE TABLE IF NOT EXISTS `users`
(
	`id` INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
	`username`              VARCHAR(255) NOT NULL UNIQUE,
	`password`              VARCHAR(255) NOT NULL,
	`permissions`           INT NOT NULL DEFAULT 0,
	`join_date`             DATETIME NOT NULL DEFAULT NOW(),
	`last_online`           DATETIME NOT NULL DEFAULT NOW(),
	`needs_password_change` TINYINT NOT NULL DEFAULT 0,
	`is_active`             TINYINT NOT NULL DEFAULT 1
);
