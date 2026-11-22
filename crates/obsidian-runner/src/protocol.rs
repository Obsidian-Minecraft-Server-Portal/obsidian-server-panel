use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

use obsidian_common::{CommonError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
	Command { line: String },
	Stop { timeout_secs: u64 },
	Kill,
	Status,
	Subscribe { backlog: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
	Starting,
	Running,
	Stopping,
	Stopped,
	Crashed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunnerMsg {
	Line { text: String },
	Status { state: RunState, pid: Option<u32>, started_at: Option<DateTime<Utc>> },
	Exited { code: Option<i32> },
}

pub fn encode<T: Serialize>(message: &T) -> Result<Vec<u8>> {
	let mut line = serde_json::to_vec(message)?;
	line.push(b'\n');
	Ok(line)
}

pub async fn write_message<W, T>(writer: &mut W, message: &T) -> Result<()>
where
	W: AsyncWrite + Unpin,
	T: Serialize,
{
	writer.write_all(&encode(message)?).await?;
	writer.flush().await?;
	Ok(())
}

pub async fn read_message<R, T>(reader: &mut R) -> Result<Option<T>>
where
	R: AsyncBufRead + Unpin,
	T: for<'de> Deserialize<'de>,
{
	let mut line = String::new();
	if reader.read_line(&mut line).await? == 0 {
		return Ok(None);
	}
	serde_json::from_str(line.trim_end())
		.map(Some)
		.map_err(|e| CommonError::Invalid(format!("malformed runner message: {e}")))
}
