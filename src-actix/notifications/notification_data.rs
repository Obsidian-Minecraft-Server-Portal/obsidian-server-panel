use serde::{Deserialize, Serialize};
use crate::database::Row;
use sqlx::{Error, FromRow, Row as _};
use sqlx::types::chrono::{DateTime, Utc};

/// Notification action types as bitflags matching the frontend enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum NotificationActionType {
    None = 0,
    AcceptDecline = 1 << 0,    // 1
    StartServer = 1 << 1,       // 2
    StopServer = 1 << 2,        // 4
    RestartServer = 1 << 3,     // 8
    ViewDetails = 1 << 4,       // 16
    UpdateNow = 1 << 5,         // 32
    ViewMessage = 1 << 6,       // 64
}

impl NotificationActionType {
    pub fn to_bits(self) -> u16 {
        self as u16
    }

    pub fn from_bits(bits: u16) -> Self {
        match bits {
            0 => Self::None,
            1 => Self::AcceptDecline,
            2 => Self::StartServer,
            4 => Self::StopServer,
            8 => Self::RestartServer,
            16 => Self::ViewDetails,
            32 => Self::UpdateNow,
            64 => Self::ViewMessage,
            _ => Self::None,
        }
    }
}

/// Notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationType {
    System,
    User,
    Action,
}

impl NotificationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Action => "action",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "user" => Self::User,
            "action" => Self::Action,
            _ => Self::System,
        }
    }
}

/// Core notification data stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationData {
    pub id: String,
    pub title: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    pub notification_type: NotificationType,
    pub action: u16, // Combined bitflags for multiple actions
    pub referenced_server: Option<String>,
}

impl<'a> FromRow<'a, Row> for NotificationData {
    fn from_row(row: &'a Row) -> Result<Self, Error> {
        Ok(NotificationData {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            message: row.try_get("message")?,
            timestamp: row.try_get("timestamp")?,
            notification_type: NotificationType::from_str(row.try_get("type")?),
            action: row.try_get::<i32, _>("action")? as u16,
            referenced_server: row.try_get("referenced_server").ok(),
        })
    }
}

/// User-specific notification state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNotification {
    pub user_id: u64,
    pub notification_id: String,
    pub is_read: bool,
    pub is_hidden: bool,
}

impl<'a> FromRow<'a, Row> for UserNotification {
    fn from_row(row: &'a Row) -> Result<Self, Error> {
        Ok(UserNotification {
            user_id: row.try_get::<u32, _>("user_id")? as u64,
            notification_id: row.try_get("notification_id")?,
            is_read: row.try_get::<i32, _>("is_read")? != 0,
            is_hidden: row.try_get::<i32, _>("is_hidden")? != 0,
        })
    }
}

/// Combined notification item with user-specific state for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationItem {
    pub id: String,
    pub title: String,
    pub message: String,
    #[serde(rename = "isRead")]
    pub is_read: bool,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    pub notification_type: NotificationType,
    pub action: u16,
    pub referenced_server: Option<String>,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationMessage {
    /// Initial list of notifications sent on connection
    InitialList { notifications: Vec<NotificationItem> },
    /// New notification created
    NewNotification { notification: NotificationItem },
    /// Notification marked as read
    MarkAsRead { id: String },
    /// All notifications marked as read
    MarkAllAsRead,
    /// Notification deleted (hidden) for this user
    DeleteNotification { id: String },
    /// All notifications deleted for this user
    DeleteAllNotifications,
    /// Server confirmation/error messages
    Error { message: String },
    Success { message: String },
}

/// Client commands sent via WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationCommand {
    /// Mark a notification as read
    MarkAsRead { id: String },
    /// Mark all notifications as read
    MarkAllAsRead,
    /// Delete (hide) a notification
    DeleteNotification { id: String },
    /// Delete all notifications for this user
    DeleteAllNotifications,
}
