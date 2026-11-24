use std::net::SocketAddr;
use std::sync::LazyLock;

use async_trait::async_trait;
use obsidian_auth::{AuthUser, Permission, generate_token, hash_password, verify_password};
use obsidian_common::{CommonError, InstanceId, Result, UserId};
use obsidian_sftp::{SftpAuth, SftpInstance, SftpUser};
use sqlx::FromRow;
use tracing::warn;

use crate::state::AppState;

static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| hash_password(&generate_token()).unwrap_or_default());

#[derive(FromRow)]
struct InstanceRow {
	id: InstanceId,
	name: String,
}

#[derive(FromRow)]
struct LoginRow {
	id: UserId,
	username: String,
	is_admin: bool,
	password_hash: String,
	totp_enabled: bool,
}

enum Verdict {
	Accepted(AuthUser),
	Rejected(Option<UserId>),
}

/// Accepts the account password, or an API token owned by `username`. Accounts with TOTP
/// enabled must use an API token, since SFTP has no second-factor step.
pub struct PanelSftpAuth {
	state: AppState,
}

impl PanelSftpAuth {
	pub fn new(state: AppState) -> Self {
		Self { state }
	}

	async fn verify(&self, username: &str, password: &str) -> Result<Verdict> {
		if let Some(user) = self.state.auth.user_for_api_token(password).await? {
			return Ok(if user.username == username { Verdict::Accepted(user) } else { Verdict::Rejected(Some(user.id)) });
		}
		let login: Option<LoginRow> =
			sqlx::query_as("SELECT id, username, is_admin, password_hash, totp_enabled FROM users WHERE username = ?")
				.bind(username)
				.fetch_optional(self.state.db.pool())
				.await?;
		let hash = login.as_ref().map(|login| login.password_hash.clone());
		let password = password.to_owned();
		let valid = tokio::task::spawn_blocking(move || verify_password(&password, hash.as_deref().unwrap_or(&DUMMY_HASH)))
			.await
			.map_err(|e| CommonError::Other(e.to_string()))?;
		Ok(match login {
			Some(login) if valid && !login.totp_enabled => {
				Verdict::Accepted(AuthUser { id: login.id, username: login.username, is_admin: login.is_admin })
			}
			other => Verdict::Rejected(other.map(|login| login.id)),
		})
	}

	async fn instances(&self, user: &AuthUser) -> Result<Vec<SftpInstance>> {
		let rows: Vec<InstanceRow> = sqlx::query_as("SELECT id, name FROM instances ORDER BY id")
			.fetch_all(self.state.db.pool())
			.await?;
		let mut instances = Vec::with_capacity(rows.len());
		for row in rows {
			let Ok(perms) = self.state.auth.instance_permission(user, row.id).await else { continue };
			if perms.contains(Permission::FILES_SFTP) {
				instances.push(SftpInstance {
					id: row.id,
					root: self.state.paths.instance_dir(row.id),
					name: row.name,
					perms: perms.bits(),
				});
			}
		}
		Ok(instances)
	}

	async fn record(&self, user_id: Option<UserId>, kind: &str, ip: Option<&str>) {
		if let Err(error) = self.state.auth.record_event(user_id, kind, ip, None).await {
			warn!(%error, "failed to record sftp security event");
		}
	}
}

#[async_trait]
impl SftpAuth for PanelSftpAuth {
	async fn authenticate(&self, username: &str, password: &str, peer: Option<SocketAddr>) -> Option<SftpUser> {
		let ip = peer.map(|peer| peer.ip().to_string());
		let user = match self.verify(username, password).await {
			Ok(Verdict::Accepted(user)) => user,
			Ok(Verdict::Rejected(user_id)) => {
				self.record(user_id, "sftp_login_failed", ip.as_deref()).await;
				return None;
			}
			Err(error) => {
				warn!(%error, "sftp authentication error");
				return None;
			}
		};
		let instances = match self.instances(&user).await {
			Ok(instances) => instances,
			Err(error) => {
				warn!(%error, "failed to load sftp instances");
				return None;
			}
		};
		self.record(Some(user.id), "sftp_login", ip.as_deref()).await;
		Some(SftpUser { user_id: user.id, instances })
	}
}
