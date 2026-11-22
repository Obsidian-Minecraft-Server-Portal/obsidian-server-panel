use std::process::Stdio;
use std::sync::Arc;

use chrono::Utc;
use interprocess::local_socket::ListenerOptions;
use obsidian_common::{CommonError, InstanceId, Paths, Result};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::client::RunnerClient;
use crate::config::RunnerConfig;
use crate::protocol::RunState;
use crate::server;
use crate::shared::{Control, Shared, Status};
use crate::socket;

const CONTROL_QUEUE: usize = 8;
const EXIT_GRACE: std::time::Duration = std::time::Duration::from_millis(500);

pub async fn supervise(paths: &Paths, id: InstanceId, config: RunnerConfig) -> Result<()> {
	let pid_file = paths.runners_dir.join(format!("{id}.pid"));
	let socket = socket::path(paths, id);
	if RunnerClient::connect(paths, id).await.is_ok() {
		return Err(CommonError::Conflict(format!("a supervisor for instance {id} is already running")));
	}
	let listener = ListenerOptions::new().name(socket::name(socket.clone())?).try_overwrite(true).create_tokio()?;
	tokio::fs::write(&pid_file, std::process::id().to_string()).await?;

	let (control_tx, mut control_rx) = mpsc::channel(CONTROL_QUEUE);
	let shared = Arc::new(Shared::new(control_tx));
	let server = tokio::spawn(server::accept_loop(listener, Arc::clone(&shared)));

	let outcome = run_server(&config, Arc::clone(&shared), &mut control_rx).await;
	let code = match &outcome {
		Ok(code) => *code,
		Err(error) => {
			tracing::error!(%error, "failed to run the server process");
			None
		}
	};
	shared.finish(code);
	tokio::time::sleep(EXIT_GRACE).await;

	server.abort();
	let _ = tokio::fs::remove_file(&socket).await;
	let _ = tokio::fs::remove_file(&pid_file).await;
	outcome.map(|_| ())
}

async fn run_server(
	config: &RunnerConfig,
	shared: Arc<Shared>,
	control: &mut mpsc::Receiver<Control>,
) -> Result<Option<i32>> {
	let mut child = Command::new(&config.java)
		.args(&config.jvm_args)
		.args(&config.args)
		.current_dir(&config.cwd)
		.envs(&config.env)
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.kill_on_drop(true)
		.spawn()?;

	shared.attach_stdin(child.stdin.take()).await;
	shared.set_status(Status { state: RunState::Starting, pid: child.id(), started_at: Some(Utc::now()) });
	let stdout = tokio::spawn(pump(child.stdout.take(), Arc::clone(&shared)));
	let stderr = tokio::spawn(pump(child.stderr.take(), Arc::clone(&shared)));

	let status = loop {
		tokio::select! {
			status = child.wait() => break status?,
			Some(Control::Kill) = control.recv() => {
				let _ = child.start_kill();
			}
		}
	};
	shared.detach_stdin().await;
	let _ = tokio::join!(stdout, stderr);
	Ok(status.code())
}

async fn pump<R: AsyncRead + Unpin>(reader: Option<R>, shared: Arc<Shared>) {
	let Some(reader) = reader else { return };
	let mut reader = BufReader::new(reader);
	let mut buffer = Vec::new();
	loop {
		buffer.clear();
		match reader.read_until(b'\n', &mut buffer).await {
			Ok(0) | Err(_) => break,
			Ok(_) => shared.push_line(String::from_utf8_lossy(&buffer).trim_end().to_owned()),
		}
	}
}
