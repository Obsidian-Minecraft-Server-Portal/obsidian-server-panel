use crate::database::{Pool, sql};
use crate::notifications::notification_data::{NotificationData, NotificationItem, NotificationType};
use anyhow::Result;
use log::debug;
use sqlx::{Executor, Row as _};
use uuid::Uuid;

#[cfg(feature = "sqlite")]
static CREATE_NOTIFICATIONS_TABLE_SQL: &str = include_str!("../../resources/sql/sqlite/notifications.sql");
#[cfg(feature = "mysql")]
static CREATE_NOTIFICATIONS_TABLE_SQL: &str = include_str!("../../resources/sql/mysql/notifications.sql");
#[cfg(feature = "postgres")]
static CREATE_NOTIFICATIONS_TABLE_SQL: &str = include_str!("../../resources/sql/postgres/notifications.sql");

/// Initialize the notifications tables
pub async fn initialize(pool: &Pool) -> Result<()> {
    debug!("Initializing notifications database...");
    pool.execute(CREATE_NOTIFICATIONS_TABLE_SQL).await?;
    Ok(())
}

impl NotificationData {
    /// Create a new notification and associate it with all users
    pub async fn create(
        title: impl Into<String>,
        message: impl Into<String>,
        notification_type: NotificationType,
        action: u16,
        referenced_server: Option<String>,
        pool: &Pool,
    ) -> Result<Self> {
        let id = Uuid::new_v4().to_string();
        let title = title.into();
        let message = message.into();
        let timestamp = chrono::Utc::now();

        sqlx::query(
            &*sql(r#"INSERT INTO notifications (id, title, message, timestamp, type, action, referenced_server)
               VALUES (?, ?, ?, ?, ?, ?, ?)"#),
        )
        .bind(&id)
        .bind(&title)
        .bind(&message)
        .bind(timestamp)
        .bind(notification_type.as_str())
        .bind(action as i32)
        .bind(&referenced_server)
        .execute(pool)
        .await?;

        // Get all users and create user_notification entries
        let user_ids = sqlx::query_scalar::<_, i64>(&*sql("SELECT id FROM users"))
            .fetch_all(pool)
            .await?;

        for user_id in user_ids {
            sqlx::query(
                &*sql(r#"INSERT INTO user_notifications (user_id, notification_id, is_read, is_hidden)
                   VALUES (?, ?, 0, 0)"#),
            )
            .bind(user_id)
            .bind(&id)
            .execute(pool)
            .await?;
        }

        Ok(NotificationData {
            id,
            title,
            message,
            timestamp,
            notification_type,
            action,
            referenced_server,
        })
    }

    /// Get all notifications for a specific user with their read/hidden state
    pub async fn get_for_user(user_id: u64, pool: &Pool) -> Result<Vec<NotificationItem>> {
        let notifications = sqlx::query(
            &*sql(r#"SELECT n.id, n.title, n.message, n.timestamp, n.type, n.action, n.referenced_server,
                      un.is_read, un.is_hidden
               FROM notifications n
               INNER JOIN user_notifications un ON n.id = un.notification_id
               WHERE un.user_id = ? AND un.is_hidden = 0
               ORDER BY n.timestamp DESC"#),
        )
        .bind(user_id as i64)
        .fetch_all(pool)
        .await?;

        let items = notifications
            .iter()
            .map(|row| {
                Ok(NotificationItem {
                    id: row.try_get("id")?,
                    title: row.try_get("title")?,
                    message: row.try_get("message")?,
                    is_read: row.try_get::<i32, _>("is_read")? != 0,
                    timestamp: row.try_get("timestamp")?,
                    notification_type: NotificationType::from_str(row.try_get("type")?),
                    action: row.try_get::<i32, _>("action")? as u16,
                    referenced_server: row.try_get("referenced_server").ok(),
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;

        Ok(items)
    }

    /// Mark a notification as read for a specific user
    pub async fn mark_as_read(notification_id: &str, user_id: u64, pool: &Pool) -> Result<()> {
        sqlx::query(
            &*sql(r#"UPDATE user_notifications
               SET is_read = 1
               WHERE notification_id = ? AND user_id = ?"#),
        )
        .bind(notification_id)
        .bind(user_id as i64)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Mark all notifications as read for a specific user
    pub async fn mark_all_as_read(user_id: u64, pool: &Pool) -> Result<()> {
        sqlx::query(&*sql(r#"UPDATE user_notifications SET is_read = 1 WHERE user_id = ?"#))
            .bind(user_id as i64)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Hide (delete) a notification for a specific user
    pub async fn hide_for_user(notification_id: &str, user_id: u64, pool: &Pool) -> Result<()> {
        sqlx::query(
            &*sql(r#"UPDATE user_notifications
               SET is_hidden = 1
               WHERE notification_id = ? AND user_id = ?"#),
        )
        .bind(notification_id)
        .bind(user_id as i64)
        .execute(pool)
        .await?;

        // Check if all users have hidden this notification
        let hidden_count: i64 = sqlx::query_scalar(
            &*sql(r#"SELECT COUNT(*) FROM user_notifications
               WHERE notification_id = ? AND is_hidden = 1"#),
        )
        .bind(notification_id)
        .fetch_one(pool)
        .await?;

        let total_count: i64 = sqlx::query_scalar(
            &*sql(r#"SELECT COUNT(*) FROM user_notifications
               WHERE notification_id = ?"#),
        )
        .bind(notification_id)
        .fetch_one(pool)
        .await?;

        // If all users have hidden it, delete the notification entirely
        if hidden_count == total_count {
            sqlx::query(&*sql(r#"DELETE FROM notifications WHERE id = ?"#))
                .bind(notification_id)
                .execute(pool)
                .await?;
        }

        Ok(())
    }

    /// Hide all notifications for a specific user
    pub async fn hide_all_for_user(user_id: u64, pool: &Pool) -> Result<()> {
        sqlx::query(&*sql(r#"UPDATE user_notifications SET is_hidden = 1 WHERE user_id = ?"#))
            .bind(user_id as i64)
            .execute(pool)
            .await?;

        // Clean up notifications that all users have hidden
        sqlx::query(
            r#"DELETE FROM notifications
               WHERE id IN (
                   SELECT notification_id
                   FROM user_notifications
                   GROUP BY notification_id
                   HAVING COUNT(*) = SUM(CASE WHEN is_hidden = 1 THEN 1 ELSE 0 END)
               )"#,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Associate a notification with a newly created user
    pub async fn associate_with_new_user(user_id: u64, pool: &Pool) -> Result<()> {
        // Get all existing notifications
        let notification_ids = sqlx::query_scalar::<_, String>("SELECT id FROM notifications")
            .fetch_all(pool)
            .await?;

        // Create user_notification entries for all existing notifications
        for notification_id in notification_ids {
            sqlx::query(
                &*sql(r#"INSERT INTO user_notifications (user_id, notification_id, is_read, is_hidden)
                   VALUES (?, ?, 0, 0)"#),
            )
            .bind(user_id as i64)
            .bind(&notification_id)
            .execute(pool)
            .await?;
        }

        Ok(())
    }
}
