-- Notifications table: stores the core notification data
CREATE TABLE IF NOT EXISTS `notifications` (
	`id` VARCHAR(255) PRIMARY KEY NOT NULL,
	`title` VARCHAR(500) NOT NULL,
	`message` TEXT NOT NULL,
	`timestamp` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
	`type` VARCHAR(20) NOT NULL CHECK(`type` IN ('system', 'user', 'action')),
	`action` INT NOT NULL DEFAULT 0,
	`referenced_server` VARCHAR(255),
	INDEX `idx_notifications_timestamp` (`timestamp` DESC)
);

-- User notifications table: tracks per-user read/hidden state
CREATE TABLE IF NOT EXISTS `user_notifications` (
	`user_id` INT UNSIGNED NOT NULL,
	`notification_id` VARCHAR(255) NOT NULL,
	`is_read` TINYINT NOT NULL DEFAULT 0,
	`is_hidden` TINYINT NOT NULL DEFAULT 0,
	PRIMARY KEY (`user_id`, `notification_id`),
	FOREIGN KEY (`user_id`) REFERENCES `users`(`id`) ON DELETE CASCADE,
	FOREIGN KEY (`notification_id`) REFERENCES `notifications`(`id`) ON DELETE CASCADE,
	INDEX `idx_user_notifications_user_id` (`user_id`),
	INDEX `idx_user_notifications_notification_id` (`notification_id`)
);
