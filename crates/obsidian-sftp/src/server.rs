use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use obsidian_common::{CommonError, Result};
use russh::keys::{Algorithm, PrivateKey, ssh_key::LineEnding};
use russh::server::{Auth, Config, Handler, Msg, Server as _, Session};
use russh::{Channel, ChannelId, MethodKind, MethodSet};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tracing::{debug, warn};

use crate::auth::{SftpAuth, SftpUser};
use crate::session::SftpSession;

const SFTP_SUBSYSTEM: &str = "sftp";
const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(600);

#[derive(Debug)]
pub struct SftpServer {
	addr: SocketAddr,
	task: JoinHandle<()>,
}

impl SftpServer {
	/// Serves SFTP on `addr`, creating the Ed25519 host key at `host_key` on first start.
	pub async fn start(addr: SocketAddr, host_key: impl AsRef<Path>, auth: Arc<dyn SftpAuth>) -> Result<Self> {
		let key = load_or_create_host_key(host_key.as_ref()).await?;
		let listener = TcpListener::bind(addr).await?;
		let addr = listener.local_addr()?;
		let config = Arc::new(Config {
			keys: vec![key],
			methods: MethodSet::from(&[MethodKind::Password][..]),
			inactivity_timeout: Some(INACTIVITY_TIMEOUT),
			..Config::default()
		});
		let mut server = SshServer { auth };
		let task = tokio::spawn(async move {
			if let Err(error) = server.run_on_socket(config, &listener).await {
				warn!(%error, "sftp server stopped");
			}
		});
		Ok(Self { addr, task })
	}

	pub fn addr(&self) -> SocketAddr {
		self.addr
	}

	pub fn stop(&self) {
		self.task.abort();
	}
}

async fn load_or_create_host_key(path: &Path) -> Result<PrivateKey> {
	match tokio::fs::read_to_string(path).await {
		Ok(pem) => PrivateKey::from_openssh(pem).map_err(|e| CommonError::Other(format!("invalid sftp host key: {e}"))),
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
			let key = PrivateKey::random(&mut rand::rng(), Algorithm::Ed25519)
				.map_err(|e| CommonError::Other(format!("could not generate sftp host key: {e}")))?;
			let pem = key
				.to_openssh(LineEnding::LF)
				.map_err(|e| CommonError::Other(format!("could not encode sftp host key: {e}")))?;
			let mut options = tokio::fs::OpenOptions::new();
			options.write(true).create_new(true);
			#[cfg(unix)]
			options.mode(0o600);
			options.open(path).await?.write_all(pem.as_bytes()).await?;
			Ok(key)
		}
		Err(error) => Err(error.into()),
	}
}

#[derive(Clone)]
struct SshServer {
	auth: Arc<dyn SftpAuth>,
}

impl russh::server::Server for SshServer {
	type Handler = SshSession;

	fn new_client(&mut self, peer: Option<SocketAddr>) -> SshSession {
		debug!(?peer, "sftp client connected");
		SshSession { auth: Arc::clone(&self.auth), peer, user: None, channel: None }
	}
}

struct SshSession {
	auth: Arc<dyn SftpAuth>,
	peer: Option<SocketAddr>,
	user: Option<SftpUser>,
	channel: Option<Channel<Msg>>,
}

impl Handler for SshSession {
	type Error = russh::Error;

	async fn auth_password(&mut self, user: &str, password: &str) -> std::result::Result<Auth, Self::Error> {
		match self.auth.authenticate(user, password, self.peer).await {
			Some(sftp_user) => {
				self.user = Some(sftp_user);
				Ok(Auth::Accept)
			}
			None => Ok(Auth::reject()),
		}
	}

	async fn channel_open_session(
		&mut self,
		channel: Channel<Msg>,
		reply: russh::server::ChannelOpenHandle,
		_session: &mut Session,
	) -> std::result::Result<(), Self::Error> {
		self.channel = Some(channel);
		reply.accept().await;
		Ok(())
	}

	async fn subsystem_request(
		&mut self,
		channel_id: ChannelId,
		name: &str,
		session: &mut Session,
	) -> std::result::Result<(), Self::Error> {
		let channel = self.channel.take();
		let user = self.user.clone();
		match (name == SFTP_SUBSYSTEM, channel, user) {
			(true, Some(channel), Some(user)) => {
				session.channel_success(channel_id)?;
				russh_sftp::server::run(channel.into_stream(), SftpSession::new(user)).await;
			}
			_ => session.channel_failure(channel_id)?,
		}
		Ok(())
	}
}

