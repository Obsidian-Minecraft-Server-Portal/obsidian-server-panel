use interprocess::local_socket::tokio::Stream;
use interprocess::local_socket::traits::tokio::Stream as _;
use obsidian_common::{InstanceId, Paths, Result};
use tokio::io::{BufReader, ReadHalf, WriteHalf};
use tokio::sync::mpsc;

use crate::protocol::{ClientMsg, RunnerMsg, read_message, write_message};
use crate::socket;

const CHANNEL_CAPACITY: usize = 256;

pub struct RunnerClient {
	reader: BufReader<ReadHalf<Stream>>,
	writer: WriteHalf<Stream>,
}

impl RunnerClient {
	pub async fn connect(paths: &Paths, id: InstanceId) -> Result<Self> {
		let name = socket::name(socket::path(paths, id))?;
		let (reader, writer) = tokio::io::split(Stream::connect(name).await?);
		Ok(Self { reader: BufReader::new(reader), writer })
	}

	pub async fn send(&mut self, message: &ClientMsg) -> Result<()> {
		write_message(&mut self.writer, message).await
	}

	pub async fn subscribe(mut self, backlog: bool) -> Result<mpsc::Receiver<RunnerMsg>> {
		self.send(&ClientMsg::Subscribe { backlog }).await?;
		let (tx, rx) = mpsc::channel(CHANNEL_CAPACITY);
		tokio::spawn(async move {
			while let Ok(Some(message)) = read_message::<_, RunnerMsg>(&mut self.reader).await {
				if tx.send(message).await.is_err() {
					break;
				}
			}
		});
		Ok(rx)
	}
}
