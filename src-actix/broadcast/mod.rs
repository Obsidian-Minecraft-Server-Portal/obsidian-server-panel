pub mod broadcast_data;
pub mod updates_endpoint;

use broadcast_data::BroadcastMessage;
use std::sync::LazyLock;
use tokio::sync::broadcast;

/// Global broadcast channel for sending updates to all connected WebSocket clients
/// Buffer size of 1000 messages - if a slow client falls behind, older messages will be dropped
static BROADCAST_CHANNEL: LazyLock<broadcast::Sender<BroadcastMessage>> =
    LazyLock::new(|| broadcast::channel(1000).0);

/// Send a message to all connected WebSocket clients
/// Returns the number of receivers that received the message
pub fn broadcast(message: BroadcastMessage) -> usize {
    BROADCAST_CHANNEL.send(message).unwrap_or(0)
}

/// Subscribe to the broadcast channel to receive updates
/// Returns a receiver that can be used to receive messages
pub fn subscribe() -> broadcast::Receiver<BroadcastMessage> {
    BROADCAST_CHANNEL.subscribe()
}
