//! The Source RCON protocol that Minecraft serves. Minecraft answers each command with one response packet, and the
//! client waits for the packet that carries the same request id.

use std::net::SocketAddr;
use std::time::Duration;

use obsidian_common::{CommonError, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

const AUTH: i32 = 3;
const EXEC: i32 = 2;
const AUTH_RESPONSE: i32 = 2;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const READ_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_PACKET: usize = 1 << 20;
/// A body this long may continue in a further packet with the same id.
const CONTINUED_BODY: usize = 4000;
const CONTINUATION_TIMEOUT: Duration = Duration::from_millis(100);

pub struct RconClient {
	stream: TcpStream,
	next_id: i32,
}

impl RconClient {
	pub async fn connect(port: u16, password: &str) -> Result<Self> {
		let address = SocketAddr::from(([127, 0, 0, 1], port));
		let stream = timeout(CONNECT_TIMEOUT, TcpStream::connect(address))
			.await
			.map_err(|_| rcon_error("the server did not accept the connection"))?
			.map_err(rcon_error)?;
		let mut client = Self { stream, next_id: 1 };
		let id = client.allocate_id();
		client.send(id, AUTH, password).await?;
		loop {
			let (reply, kind, _) = client.receive().await?;
			if kind == AUTH_RESPONSE {
				return if reply == id {
					Ok(client)
				} else {
					Err(rcon_error("the password was rejected"))
				};
			}
		}
	}

	pub async fn command(&mut self, command: &str) -> Result<String> {
		let id = self.allocate_id();
		self.send(id, EXEC, command).await?;
		let mut body = String::new();
		loop {
			let (reply, _, chunk) = self.receive().await?;
			if reply != id {
				continue;
			}
			let continues = chunk.len() >= CONTINUED_BODY;
			body.push_str(&chunk);
			if !continues {
				return Ok(body);
			}
			match timeout(CONTINUATION_TIMEOUT, self.receive()).await {
				Ok(Ok((next, _, more))) if next == id => body.push_str(&more),
				_ => return Ok(body),
			}
		}
	}

	fn allocate_id(&mut self) -> i32 {
		let id = self.next_id;
		self.next_id = self.next_id.wrapping_add(1).max(1);
		id
	}

	async fn send(&mut self, id: i32, kind: i32, body: &str) -> Result<()> {
		let mut packet = Vec::with_capacity(body.len() + 14);
		packet.extend_from_slice(&((body.len() + 10) as i32).to_le_bytes());
		packet.extend_from_slice(&id.to_le_bytes());
		packet.extend_from_slice(&kind.to_le_bytes());
		packet.extend_from_slice(body.as_bytes());
		packet.extend_from_slice(&[0, 0]);
		timeout(READ_TIMEOUT, self.stream.write_all(&packet))
			.await
			.map_err(|_| rcon_error("sending timed out"))?
			.map_err(rcon_error)
	}

	async fn receive(&mut self) -> Result<(i32, i32, String)> {
		let mut length = [0_u8; 4];
		self.read(&mut length).await?;
		let length = i32::from_le_bytes(length);
		if !(10..=MAX_PACKET as i32).contains(&length) {
			return Err(rcon_error("the server sent a malformed packet"));
		}
		let mut payload = vec![0_u8; length as usize];
		self.read(&mut payload).await?;
		let id = i32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
		let kind = i32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
		let body = String::from_utf8_lossy(&payload[8..payload.len() - 2]).into_owned();
		Ok((id, kind, body))
	}

	async fn read(&mut self, buffer: &mut [u8]) -> Result<()> {
		timeout(READ_TIMEOUT, self.stream.read_exact(buffer))
			.await
			.map_err(|_| rcon_error("the server stopped answering"))?
			.map(|_| ())
			.map_err(rcon_error)
	}
}

fn rcon_error(error: impl std::fmt::Display) -> CommonError {
	CommonError::Other(format!("RCON: {error}"))
}

#[cfg(test)]
mod tests {
	use tokio::io::{AsyncReadExt, AsyncWriteExt};
	use tokio::net::TcpListener;

	use super::*;

	async fn packet(stream: &mut TcpStream) -> (i32, i32, String) {
		let mut length = [0_u8; 4];
		stream.read_exact(&mut length).await.unwrap();
		let mut payload = vec![0_u8; i32::from_le_bytes(length) as usize];
		stream.read_exact(&mut payload).await.unwrap();
		let id = i32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]);
		let kind = i32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]);
		(id, kind, String::from_utf8_lossy(&payload[8..payload.len() - 2]).into_owned())
	}

	async fn reply(stream: &mut TcpStream, id: i32, kind: i32, body: &str) {
		let mut bytes = ((body.len() + 10) as i32).to_le_bytes().to_vec();
		bytes.extend_from_slice(&id.to_le_bytes());
		bytes.extend_from_slice(&kind.to_le_bytes());
		bytes.extend_from_slice(body.as_bytes());
		bytes.extend_from_slice(&[0, 0]);
		stream.write_all(&bytes).await.unwrap();
	}

	/// A server that accepts only `secret` and answers every command with its own text.
	async fn fake_server() -> u16 {
		let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
		let port = listener.local_addr().unwrap().port();
		tokio::spawn(async move {
			let (mut stream, _) = listener.accept().await.unwrap();
			let (id, kind, password) = packet(&mut stream).await;
			assert_eq!(kind, AUTH);
			let accepted = password == "secret";
			reply(&mut stream, id, 0, "").await;
			reply(&mut stream, if accepted { id } else { -1 }, AUTH_RESPONSE, "").await;
			if !accepted {
				return;
			}
			loop {
				let (id, kind, command) = packet(&mut stream).await;
				if kind == EXEC {
					reply(&mut stream, id, 0, &format!("ran: {command}")).await;
				}
			}
		});
		port
	}

	#[tokio::test]
	async fn runs_commands_after_authenticating() {
		let port = fake_server().await;
		let mut client = RconClient::connect(port, "secret").await.unwrap();
		assert_eq!(client.command("list").await.unwrap(), "ran: list");
		assert_eq!(client.command("data get entity @a Pos").await.unwrap(), "ran: data get entity @a Pos");
	}

	#[tokio::test]
	async fn refuses_a_wrong_password() {
		let port = fake_server().await;
		let error = RconClient::connect(port, "guess").await.err().expect("authentication fails");
		assert!(error.to_string().contains("rejected"), "{error}");
	}
}
