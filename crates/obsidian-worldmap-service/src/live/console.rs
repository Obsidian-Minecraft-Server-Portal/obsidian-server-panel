//! Asks questions through the server console. A command is typed as an operator would type it, and its replies are the
//! console lines the server prints next. Those lines are hidden from the console tab while the poller runs.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use obsidian_common::{InstanceId, Result};
use obsidian_instances::{ConsoleFilter, InstanceManager};
use obsidian_worldmap::players::is_poll_reply;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::{Instant, timeout_at};

use super::now_millis;

/// Replies stay hidden this long after each command, which is ample for the server to answer it.
const HIDE_FOR_MS: i64 = 3_000;

pub struct Console {
	instances: InstanceManager,
	id: InstanceId,
	hide_until: Arc<AtomicI64>,
}

impl Console {
	pub fn new(instances: InstanceManager, id: InstanceId) -> Self {
		Self {
			instances,
			id,
			hide_until: Arc::new(AtomicI64::new(0)),
		}
	}

	pub fn hide_replies(&self) {
		let hide_until = Arc::clone(&self.hide_until);
		let filter = ConsoleFilter::new(move |line| now_millis() < hide_until.load(Ordering::Relaxed) && is_poll_reply(line));
		self.instances.set_console_filter(self.id, Some(filter));
	}

	pub fn show_replies(&self) {
		self.instances.set_console_filter(self.id, None);
	}

	/// Sends `command`, then collects the console lines that answer it until `done` is satisfied or `wait` runs out.
	pub async fn ask(&self, command: &str, wait: Duration, mut done: impl FnMut(&[String]) -> bool) -> Result<Vec<String>> {
		let mut lines = self.instances.console_lines();
		self.hide_until.store(now_millis() + HIDE_FOR_MS, Ordering::Relaxed);
		self.instances.send_command(self.id, command).await?;
		let deadline = Instant::now() + wait;
		let mut replies = Vec::new();
		while let Ok(received) = timeout_at(deadline, lines.recv()).await {
			match received {
				Ok(line) if line.instance == self.id && is_poll_reply(&line.text) => {
					replies.push(line.text);
					if done(&replies) {
						break;
					}
				}
				Ok(_) | Err(RecvError::Lagged(_)) => {}
				Err(RecvError::Closed) => break,
			}
		}
		Ok(replies)
	}
}

