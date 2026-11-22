use std::collections::VecDeque;

use obsidian_common::InstanceId;
use obsidian_runner::RunState;

use crate::model::{InstanceStats, InstanceStatus};

pub const CONSOLE_LINES: usize = 1000;

#[derive(Debug, Default)]
pub struct Live {
	pub status: InstanceStatus,
	pub port: u16,
	pub console: VecDeque<String>,
	pub stats: InstanceStats,
	pub upnp_port: Option<u16>,
}

impl Live {
	pub fn push_console(&mut self, line: String) {
		if self.console.len() == CONSOLE_LINES {
			self.console.pop_front();
		}
		self.console.push_back(line);
	}

	pub fn tail(&self, lines: usize) -> Vec<String> {
		let skip = self.console.len().saturating_sub(lines);
		self.console.iter().skip(skip).cloned().collect()
	}
}

pub fn is_active(state: RunState) -> bool {
	matches!(state, RunState::Starting | RunState::Running | RunState::Stopping)
}

pub fn console_topic(id: InstanceId) -> String {
	format!("instance.{id}.console")
}

pub fn status_topic(id: InstanceId) -> String {
	format!("instance.{id}.status")
}

pub fn stats_topic(id: InstanceId) -> String {
	format!("instance.{id}.stats")
}
