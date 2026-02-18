-- Notifications table: stores the core notification data
CREATE TABLE IF NOT EXISTS notifications (
	id                VARCHAR(255) PRIMARY KEY NOT NULL,
	title             VARCHAR(500) NOT NULL,
	message           TEXT NOT NULL,
	timestamp         TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	type              VARCHAR(20) NOT NULL CHECK(type IN ('system', 'user', 'action')),
	action            INT NOT NULL DEFAULT 0,
	referenced_server VARCHAR(255)
);
CREATE INDEX IF NOT EXISTS idx_notifications_timestamp ON notifications(timestamp DESC);

-- User notifications table: tracks per-user read/hidden state
CREATE TABLE IF NOT EXISTS user_notifications (
	user_id         INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	notification_id VARCHAR(255) NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
	is_read         SMALLINT NOT NULL DEFAULT 0,
	is_hidden       SMALLINT NOT NULL DEFAULT 0,
	PRIMARY KEY (user_id, notification_id)
);
CREATE INDEX IF NOT EXISTS idx_user_notifications_user_id ON user_notifications(user_id);
CREATE INDEX IF NOT EXISTS idx_user_notifications_notification_id ON user_notifications(notification_id);
