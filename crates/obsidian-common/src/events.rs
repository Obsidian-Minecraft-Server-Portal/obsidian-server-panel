use serde_json::Value;
use tokio::sync::broadcast;

use crate::UserId;

const CHANNEL_CAPACITY: usize = 1024;

#[derive(Debug, Clone)]
pub struct Event {
	pub topic: String,
	pub data: Value,
	pub user: Option<UserId>,
}

#[derive(Debug, Clone)]
pub struct EventBus {
	tx: broadcast::Sender<Event>,
}

impl EventBus {
	pub fn new() -> Self {
		let (tx, _) = broadcast::channel(CHANNEL_CAPACITY);
		Self { tx }
	}

	pub fn publish(&self, topic: impl Into<String>, data: Value) {
		self.send(topic.into(), data, None);
	}

	pub fn publish_to(&self, user: UserId, topic: impl Into<String>, data: Value) {
		self.send(topic.into(), data, Some(user));
	}

	pub fn subscribe(&self) -> broadcast::Receiver<Event> {
		self.tx.subscribe()
	}

	fn send(&self, topic: String, data: Value, user: Option<UserId>) {
		let _ = self.tx.send(Event { topic, data, user });
	}
}

impl Default for EventBus {
	fn default() -> Self {
		Self::new()
	}
}
