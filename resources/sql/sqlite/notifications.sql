-- Notifications table: stores the core notification data
CREATE TABLE IF NOT EXISTS notifications (
	id                TEXT PRIMARY KEY NOT NULL,
	title             TEXT NOT NULL,
	message           TEXT NOT NULL,
	timestamp         TEXT NOT NULL DEFAULT (DATETIME('now')),
	type              TEXT NOT NULL CHECK(type IN ('system', 'user', 'action')),
	action            INTEGER NOT NULL DEFAULT 0,
	referenced_server TEXT
);
CREATE INDEX IF NOT EXISTS idx_notifications_timestamp ON notifications(timestamp DESC);

-- User notifications table: tracks per-user read/hidden state
CREATE TABLE IF NOT EXISTS user_notifications (
	user_id         INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	notification_id TEXT NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
	is_read         INTEGER NOT NULL DEFAULT 0,
	is_hidden       INTEGER NOT NULL DEFAULT 0,
	PRIMARY KEY (user_id, notification_id)
);
CREATE INDEX IF NOT EXISTS idx_user_notifications_user_id ON user_notifications(user_id);
CREATE INDEX IF NOT EXISTS idx_user_notifications_notification_id ON user_notifications(notification_id);
