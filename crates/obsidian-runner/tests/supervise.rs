use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use obsidian_common::Paths;
use obsidian_runner::{ClientMsg, DATA_DIR_ENV, RunState, RunnerClient, RunnerConfig, RunnerMsg};
use tokio::sync::mpsc::Receiver;

const CHILD_FLAG: &str = "OBSIDIAN_RUNNER_TEST_CHILD";
const INSTANCE: i64 = 4242;

#[test]
fn echo_child() {
	if std::env::var_os(CHILD_FLAG).is_none() {
		return;
	}
	use std::io::{BufRead, Write};
	println!("Done (0.1s)");
	std::io::stdout().flush().unwrap();
	for line in std::io::stdin().lock().lines() {
		let line = line.unwrap();
		println!("got:{line}");
		std::io::stdout().flush().unwrap();
		if line == "stop" {
			break;
		}
	}
}

async fn next_until(rx: &mut Receiver<RunnerMsg>, mut matches: impl FnMut(&RunnerMsg) -> bool) -> RunnerMsg {
	tokio::time::timeout(Duration::from_secs(10), async {
		loop {
			let message = rx.recv().await.expect("runner stream closed early");
			if matches(&message) {
				return message;
			}
		}
	})
	.await
	.expect("timed out waiting for runner message")
}

#[tokio::test(flavor = "multi_thread")]
async fn runner_round_trip() {
	let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("runner-smoke");
	let _ = std::fs::remove_dir_all(&dir);
	let paths = Paths::new(&dir).expect("paths");
	let config = RunnerConfig {
		java: std::env::current_exe().expect("test binary path"),
		jvm_args: Vec::new(),
		args: vec!["--exact".into(), "echo_child".into(), "--nocapture".into()],
		cwd: dir.clone(),
		env: BTreeMap::from([(CHILD_FLAG.to_owned(), "1".to_owned())]),
	};
	config.save(&paths, INSTANCE).await.expect("save config");
	std::fs::write(paths.runners_dir.join(format!("{INSTANCE}.sock")), b"").expect("stale socket");

	// SAFETY: this is the only test in the binary and it is set before any other thread reads the variable.
	unsafe { std::env::set_var(DATA_DIR_ENV, &dir) };
	let runner = tokio::spawn(obsidian_runner::run(INSTANCE));

	let mut events = loop {
		if let Ok(client) = RunnerClient::connect(&paths, INSTANCE).await {
			break client.subscribe(true).await.expect("subscribe");
		}
		tokio::time::sleep(Duration::from_millis(50)).await;
	};
	next_until(&mut events, |m| match m {
		RunnerMsg::Line { text } => text.contains("Done ("),
		RunnerMsg::Backlog { lines } => lines.iter().any(|text| text.contains("Done (")),
		_ => false,
	})
	.await;

	let mut command = RunnerClient::connect(&paths, INSTANCE).await.expect("connect");
	command.send(&ClientMsg::Command { line: "hello".into() }).await.expect("send command");
	next_until(&mut events, |m| matches!(m, RunnerMsg::Line { text } if text == "got:hello")).await;

	let mut stop = RunnerClient::connect(&paths, INSTANCE).await.expect("connect");
	stop.send(&ClientMsg::Stop { timeout_secs: 5 }).await.expect("send stop");
	next_until(&mut events, |m| matches!(m, RunnerMsg::Status { state: RunState::Stopped, .. })).await;
	let exited = next_until(&mut events, |m| matches!(m, RunnerMsg::Exited { .. })).await;
	assert!(matches!(exited, RunnerMsg::Exited { code: Some(0) }));

	tokio::time::timeout(Duration::from_secs(10), runner).await.expect("runner exit").expect("join").expect("run");
	assert!(!RunnerConfig::path(&paths, INSTANCE).with_extension("pid").exists());
}
