use obsidian_common::{Result, UserId};

use super::AuthService;
use crate::Notification;

const LIST_LIMIT: i64 = 200;

impl AuthService {
	/// Stores a notification and pushes it to the user's live sockets on the `notifications` topic.
	pub async fn notify(&self, user_id: UserId, kind: &str, title: &str, body: &str) -> Result<Notification> {
		let notification: Notification = sqlx::query_as(
			"INSERT INTO notifications (user_id, kind, title, body) VALUES (?, ?, ?, ?) \
			 RETURNING id, kind, title, body, read_at IS NOT NULL AS read, created_at",
		)
		.bind(user_id)
		.bind(kind)
		.bind(title)
		.bind(body)
		.fetch_one(self.pool())
		.await?;
		self.bus
			.publish_to(user_id, "notifications", serde_json::to_value(&notification)?);
		Ok(notification)
	}

	pub async fn list_notifications(&self, user_id: UserId) -> Result<Vec<Notification>> {
		Ok(sqlx::query_as(
			"SELECT id, kind, title, body, read_at IS NOT NULL AS read, created_at FROM notifications \
			 WHERE user_id = ? ORDER BY id DESC LIMIT ?",
		)
		.bind(user_id)
		.bind(LIST_LIMIT)
		.fetch_all(self.pool())
		.await?)
	}

	/// Marks the given notifications read, or every unread one when `ids` is `None`.
	pub async fn mark_notifications_read(&self, user_id: UserId, ids: Option<&[i64]>) -> Result<()> {
		match ids {
			None => {
				sqlx::query("UPDATE notifications SET read_at = unixepoch() WHERE user_id = ? AND read_at IS NULL")
					.bind(user_id)
					.execute(self.pool())
					.await?;
			}
			Some(ids) => {
				sqlx::query(
					"UPDATE notifications SET read_at = unixepoch() \
					 WHERE user_id = ? AND read_at IS NULL AND id IN (SELECT value FROM json_each(?))",
				)
				.bind(user_id)
				.bind(serde_json::to_string(ids)?)
				.execute(self.pool())
				.await?;
			}
		}
		Ok(())
	}
}
