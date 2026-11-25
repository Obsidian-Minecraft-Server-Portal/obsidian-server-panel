use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;

use obsidian_common::InstanceId;
use obsidian_runner::RunState;

use crate::model::{InstanceStats, InstanceStatus};
use crate::roster::Roster;

pub const CONSOLE_LINES: usize = 1000;

/// A console line as the server printed it, before any filter.
#[derive(Debug, Clone)]
pub struct ConsoleLine {
	pub instance: InstanceId,
	pub text: String,
}

/// Keeps matching console lines out of the console tab and its topic while it is set.
#[derive(Clone)]
pub struct ConsoleFilter(Arc<dyn Fn(&str) -> bool + Send + Sync>);

impl ConsoleFilter {
	pub fn new(hides: impl Fn(&str) -> bool + Send + Sync + 'static) -> Self {
		Self(Arc::new(hides))
	}

	pub fn hides(&self, line: &str) -> bool {
		(self.0)(line)
	}
}

impl fmt::Debug for ConsoleFilter {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("ConsoleFilter")
	}
}

#[derive(Debug, Default)]
pub struct Live {
	pub status: InstanceStatus,
	pub port: u16,
	pub console: VecDeque<String>,
	pub console_filter: Option<ConsoleFilter>,
	pub stats: InstanceStats,
	pub roster: Roster,
	pub upnp_port: Option<u16>,
	pub map_upnp_ports: Vec<u16>,
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
