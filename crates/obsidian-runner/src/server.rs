use std::sync::Arc;
use std::time::Duration;

use interprocess::local_socket::tokio::{Listener, Stream};
use interprocess::local_socket::traits::tokio::Listener as _;
use obsidian_common::Result;
use tokio::io::{AsyncWriteExt, BufReader, WriteHalf};
use tokio::sync::{broadcast, mpsc};

use crate::protocol::{ClientMsg, RunnerMsg, encode, read_message, write_message};
use crate::shared::Shared;

const CLIENT_QUEUE: usize = 32;
const ACCEPT_BACKOFF: Duration = Duration::from_millis(250);

pub async fn accept_loop(listener: Listener, shared: Arc<Shared>) {
	loop {
		match listener.accept().await {
			Ok(stream) => {
				tokio::spawn(serve(stream, Arc::clone(&shared)));
			}
			Err(error) => {
				tracing::warn!(%error, "runner accept failed");
				tokio::time::sleep(ACCEPT_BACKOFF).await;
			}
		}
	}
}

async fn serve(stream: Stream, shared: Arc<Shared>) {
	let (read, mut write) = tokio::io::split(stream);
	let (tx, mut rx) = mpsc::channel::<ClientMsg>(CLIENT_QUEUE);
	let reader = tokio::spawn(async move {
		let mut reader = BufReader::new(read);
		while let Ok(Some(message)) = read_message::<_, ClientMsg>(&mut reader).await {
			if tx.send(message).await.is_err() {
				break;
			}
		}
	});

	let mut events: Option<broadcast::Receiver<RunnerMsg>> = None;
	loop {
		let outcome = tokio::select! {
			message = rx.recv() => match message {
				Some(message) => handle(message, &shared, &mut write, &mut events).await,
				None => break,
			},
			event = next_event(&mut events) => match event {
				Some(message) => write_message(&mut write, &message).await,
				None => {
					events = None;
					Ok(())
				}
			},
		};
		if let Err(error) = outcome {
			tracing::debug!(%error, "runner client connection ended");
			break;
		}
	}
	reader.abort();
}

async fn handle(
	message: ClientMsg,
	shared: &Shared,
	write: &mut WriteHalf<Stream>,
	events: &mut Option<broadcast::Receiver<RunnerMsg>>,
) -> Result<()> {
	match message {
		ClientMsg::Subscribe { backlog } => {
			let (status, lines, receiver) = shared.subscribe(backlog);
			*events = Some(receiver);
			let mut frames = encode(&status.message())?;
			for text in lines {
				frames.extend(encode(&RunnerMsg::Line { text })?);
			}
			write.write_all(&frames).await?;
			write.flush().await?;
		}
		ClientMsg::Status => write_message(write, &shared.status().message()).await?,
		ClientMsg::Command { line } => shared.send_line(&line).await,
		ClientMsg::Stop { timeout_secs } => shared.stop(Duration::from_secs(timeout_secs)).await,
		ClientMsg::Kill => shared.kill().await,
	}
	Ok(())
}

async fn next_event(events: &mut Option<broadcast::Receiver<RunnerMsg>>) -> Option<RunnerMsg> {
	let Some(receiver) = events.as_mut() else {
		return std::future::pending().await;
	};
	loop {
		match receiver.recv().await {
			Ok(message) => return Some(message),
			Err(broadcast::error::RecvError::Lagged(_)) => continue,
			Err(broadcast::error::RecvError::Closed) => return None,
		}
	}
}
