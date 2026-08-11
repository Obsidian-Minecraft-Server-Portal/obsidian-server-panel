pub mod notification_data;
mod notification_db;

use crate::broadcast::{self, broadcast_data::BroadcastMessage};
use notification_data::NotificationMessage;

pub use notification_data::{NotificationActionType, NotificationData, NotificationItem, NotificationType};
pub use notification_db::initialize;

/// Push a newly created notification to every client connected to `/api/updates/ws`.
pub async fn broadcast_notification(notification: NotificationItem) {
    broadcast::broadcast(BroadcastMessage::Notification {
        message: NotificationMessage::NewNotification { notification },
    });
}
