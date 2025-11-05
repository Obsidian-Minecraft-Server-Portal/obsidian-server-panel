use crate::actions::actions_data::ActionData;
use crate::notifications::notification_data::NotificationMessage;
use crate::server::server_data::ServerData;
use craftping::Response as PingResponse;
use serde::{Deserialize, Serialize};

/// Messages that can be broadcast to all connected WebSocket clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BroadcastMessage {
    /// Server created, updated, or status changed
    ServerUpdate {
        server: ServerData,
    },
    /// Server deleted
    ServerDeleted {
        /// Hashed server ID (using serde_hash)
        server_id: String,
    },
    /// Server ping data (player count, etc.)
    ServerPing {
        /// Hashed server ID (using serde_hash)
        server_id: String,
        ping: PingResponse,
    },
    /// Action progress update
    ActionUpdate {
        action: ActionData,
    },
    /// Action completed or failed
    ActionComplete {
        action_id: String,
    },
    /// User notification (from the old notification system)
    Notification {
        #[serde(flatten)]
        message: NotificationMessage,
    },
}
