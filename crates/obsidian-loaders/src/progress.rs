use std::collections::VecDeque;
use std::process::Stdio;
use std::time::{Duration, Instant};

use obsidian_common::{CommonError, Result, TaskHandle};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::{self, UnboundedSender};

const TAIL_LINES: usize = 20;
const LINE_SCALE: f32 = 40.0;
const REPORT_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy)]
pub struct Stage<'a> {
	task: &'a TaskHandle,
	start: f32,
	span: f32,
}

impl<'a> Stage<'a> {
	pub fn new(task: &'a TaskHandle) -> Self {
		Self { task, start: 0.0, span: 1.0 }
	}

	pub fn part(self, start: f32, span: f32) -> Self {
		Self { task: self.task, start: self.start + self.span * start, span: self.span * span }
	}

	pub fn report(self, fraction: f32, message: &str) {
		self.task.progress(self.start + self.span * fraction.clamp(0.0, 1.0), message);
	}

	pub fn task_id(self) -> u64 {
		self.task.id()
	}
}

pub async fn run_observed(mut command: Command, stage: Stage<'_>) -> Result<()> {
	let mut child = command.stdout(Stdio::piped()).stderr(Stdio::piped()).kill_on_drop(true).spawn()?;
	let (tx, mut lines) = mpsc::unbounded_channel::<String>();
	if let Some(stdout) = child.stdout.take() {
		tokio::spawn(forward(BufReader::new(stdout), tx.clone()));
	}
	if let Some(stderr) = child.stderr.take() {
		tokio::spawn(forward(BufReader::new(stderr), tx.clone()));
	}
	drop(tx);

	let mut tail = VecDeque::with_capacity(TAIL_LINES);
	let mut count = 0usize;
	let mut last_report: Option<Instant> = None;
	while let Some(line) = lines.recv().await {
		count += 1;
		if last_report.is_none_or(|at| at.elapsed() >= REPORT_INTERVAL) {
			last_report = Some(Instant::now());
			stage.report(1.0 - 1.0 / (1.0 + count as f32 / LINE_SCALE), &line);
		}
		if tail.len() == TAIL_LINES {
			tail.pop_front();
		}
		tail.push_back(line);
	}

	let status = child.wait().await?;
	if status.success() {
		return Ok(());
	}
	let output: Vec<String> = tail.into();
	Err(CommonError::Other(format!("process exited with {status}:\n{}", output.join("\n"))))
}

async fn forward<R: AsyncBufRead + Unpin>(reader: R, tx: UnboundedSender<String>) {
	let mut lines = reader.lines();
	while let Ok(Some(line)) = lines.next_line().await {
		if tx.send(line).is_err() {
			break;
		}
	}
}
