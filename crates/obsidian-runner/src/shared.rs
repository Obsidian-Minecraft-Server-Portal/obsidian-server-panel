use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::io::AsyncWriteExt;
use tokio::process::ChildStdin;
use tokio::sync::{Mutex as AsyncMutex, broadcast, mpsc};

use crate::protocol::{RunState, RunnerMsg};

pub const BACKLOG_LINES: usize = 1000;
const EVENT_CAPACITY: usize = 1024;
const READY_MARKER: &str = "Done (";
const STOP_COMMAND: &str = "stop";

#[derive(Debug, Clone, Copy)]
pub enum Control {
	Kill,
}

#[derive(Debug, Clone, Copy)]
pub struct Status {
	pub state: RunState,
	pub pid: Option<u32>,
	pub started_at: Option<DateTime<Utc>>,
}

impl Status {
	pub fn message(self) -> RunnerMsg {
		RunnerMsg::Status { state: self.state, pid: self.pid, started_at: self.started_at }
	}
}

pub struct Shared {
	lines: Mutex<VecDeque<String>>,
	status: Mutex<Status>,
	events: broadcast::Sender<RunnerMsg>,
	stdin: AsyncMutex<Option<ChildStdin>>,
	control: mpsc::Sender<Control>,
	stop_requested: AtomicBool,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
	mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Shared {
	pub fn new(control: mpsc::Sender<Control>) -> Self {
		let (events, _) = broadcast::channel(EVENT_CAPACITY);
		Self {
			lines: Mutex::new(VecDeque::with_capacity(BACKLOG_LINES)),
			status: Mutex::new(Status { state: RunState::Starting, pid: None, started_at: None }),
			events,
			stdin: AsyncMutex::new(None),
			control,
			stop_requested: AtomicBool::new(false),
		}
	}

	pub fn push_line(&self, text: String) {
		let mut lines = lock(&self.lines);
		if lines.len() == BACKLOG_LINES {
			lines.pop_front();
		}
		lines.push_back(text.clone());
		if text.contains(READY_MARKER) {
			let mut status = lock(&self.status);
			if status.state == RunState::Starting {
				status.state = RunState::Running;
				let _ = self.events.send(status.message());
			}
		}
		let _ = self.events.send(RunnerMsg::Line { text });
	}

	pub fn status(&self) -> Status {
		*lock(&self.status)
	}

	pub fn set_status(&self, status: Status) {
		let mut current = lock(&self.status);
		*current = status;
		let _ = self.events.send(status.message());
	}

	pub fn subscribe(&self, backlog: bool) -> (Status, Vec<String>, broadcast::Receiver<RunnerMsg>) {
		let lines = lock(&self.lines);
		let status = *lock(&self.status);
		let receiver = self.events.subscribe();
		let backlog = if backlog { lines.iter().cloned().collect() } else { Vec::new() };
		(status, backlog, receiver)
	}

	pub async fn attach_stdin(&self, stdin: Option<ChildStdin>) {
		*self.stdin.lock().await = stdin;
	}

	pub async fn send_line(&self, line: &str) {
		let mut guard = self.stdin.lock().await;
		let Some(stdin) = guard.as_mut() else { return };
		if let Err(error) = stdin.write_all(format!("{line}\n").as_bytes()).await {
			tracing::warn!(%error, "failed to write to server stdin");
		}
	}

	pub async fn stop(&self, timeout: Duration) {
		if !matches!(self.status().state, RunState::Starting | RunState::Running) {
			return;
		}
		self.stop_requested.store(true, Ordering::Release);
		self.set_state(RunState::Stopping);
		self.send_line(STOP_COMMAND).await;
		let control = self.control.clone();
		tokio::spawn(async move {
			tokio::time::sleep(timeout).await;
			let _ = control.send(Control::Kill).await;
		});
	}

	pub async fn kill(&self) {
		self.stop_requested.store(true, Ordering::Release);
		let _ = self.control.send(Control::Kill).await;
	}

	pub async fn detach_stdin(&self) {
		self.attach_stdin(None).await;
	}

	pub fn finish(&self, code: Option<i32>) {
		let requested = self.stop_requested.load(Ordering::Acquire);
		let state = if requested || code == Some(0) { RunState::Stopped } else { RunState::Crashed };
		self.set_status(Status { state, pid: None, started_at: None });
		let _ = self.events.send(RunnerMsg::Exited { code });
	}

	fn set_state(&self, state: RunState) {
		let mut status = lock(&self.status);
		status.state = state;
		let _ = self.events.send(status.message());
	}
}
