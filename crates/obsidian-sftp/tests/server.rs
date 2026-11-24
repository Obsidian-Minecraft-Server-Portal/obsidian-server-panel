use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use obsidian_auth::{Permission, generate_token};
use obsidian_common::UserId;
use obsidian_sftp::{SftpAuth, SftpInstance, SftpServer};
use russh::client::{self, Handle};
use russh::keys::PublicKeyOrCertificate;
use russh_sftp::client::SftpSession;
use russh_sftp::protocol::FileAttributes;

const USER: &str = "drew";
const USER_ID: UserId = 1;
static PASSWORD: LazyLock<String> = LazyLock::new(generate_token);

struct StaticAuth {
	instances: Mutex<Vec<SftpInstance>>,
}

#[async_trait]
impl SftpAuth for StaticAuth {
	async fn authenticate(&self, username: &str, password: &str, _peer: Option<SocketAddr>) -> Option<UserId> {
		(username == USER && password == PASSWORD.as_str()).then_some(USER_ID)
	}

	async fn instances(&self, user_id: UserId) -> Option<Vec<SftpInstance>> {
		(user_id == USER_ID).then(|| self.instances.lock().unwrap().clone())
	}
}

struct Client;

impl client::Handler for Client {
	type Error = russh::Error;

	async fn check_server_key(&mut self, _key: &PublicKeyOrCertificate) -> Result<bool, Self::Error> {
		Ok(true)
	}
}

async fn login(addr: SocketAddr, password: &str) -> Option<(Handle<Client>, SftpSession)> {
	let mut session = client::connect(Arc::new(client::Config::default()), addr, Client).await.ok()?;
	if !session.authenticate_password(USER, password).await.ok()?.success() {
		return None;
	}
	let channel = session.channel_open_session().await.ok()?;
	channel.request_subsystem(true, "sftp").await.ok()?;
	let sftp = SftpSession::new(channel.into_stream()).await.ok()?;
	Some((session, sftp))
}

fn instance(id: i64, name: &str, root: &Path, permissions: Permission) -> SftpInstance {
	SftpInstance { id, name: name.into(), root: root.to_path_buf(), perms: permissions.bits() }
}

#[tokio::test]
async fn serves_only_accessible_instances() {
	let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("sftp-e2e");
	let _ = std::fs::remove_dir_all(&base);
	let writable = base.join("instances").join("7");
	let readonly = base.join("instances").join("8");
	std::fs::create_dir_all(&writable).unwrap();
	std::fs::create_dir_all(&readonly).unwrap();
	std::fs::write(writable.join("hello.txt"), b"hi").unwrap();

	let full = Permission::FILES_READ | Permission::FILES_WRITE | Permission::FILES_DELETE | Permission::FILES_SFTP;
	let auth = Arc::new(StaticAuth {
		instances: Mutex::new(vec![
			instance(7, "Survival", &writable, full),
			instance(8, "Archive", &readonly, Permission::FILES_READ | Permission::FILES_SFTP),
			instance(9, "Hidden", &readonly, Permission::FILES_READ),
		]),
	});
	let server = SftpServer::start("127.0.0.1:0".parse().unwrap(), base.join("host_key"), auth.clone())
		.await
		.unwrap();

	assert!(login(server.addr(), &generate_token()).await.is_none());
	let (_session, sftp) = login(server.addr(), PASSWORD.as_str()).await.expect("valid login");

	let roots: Vec<String> = sftp.read_dir("/").await.unwrap().map(|e| e.file_name()).collect();
	assert_eq!(roots.len(), 2, "instance without files.sftp must stay hidden: {roots:?}");
	assert!(roots.contains(&"Survival-7".to_owned()));

	let fresh = base.join("instances").join("10");
	std::fs::create_dir_all(&fresh).unwrap();
	auth.instances.lock().unwrap().push(instance(10, "Fresh", &fresh, full));
	tokio::time::sleep(Duration::from_millis(2100)).await;
	let roots: Vec<String> = sftp.read_dir("/").await.unwrap().map(|e| e.file_name()).collect();
	assert!(roots.contains(&"Fresh-10".to_owned()), "instance created after login must appear: {roots:?}");
	sftp.metadata("/Fresh-10").await.expect("new instance must be addressable without reconnecting");

	let files: Vec<String> = sftp.read_dir("/Survival-7").await.unwrap().map(|e| e.file_name()).collect();
	assert_eq!(files, ["hello.txt"]);
	assert_eq!(sftp.read("/Survival-7/hello.txt").await.unwrap(), b"hi");

	sftp.write("/Survival-7/new.txt", b"data").await.unwrap();
	assert_eq!(std::fs::read(writable.join("new.txt")).unwrap(), b"data");

	sftp.create_dir("/Survival-7/world").await.unwrap();
	sftp.rename("/Survival-7/new.txt", "/Survival-7/world/moved.txt").await.unwrap();
	assert!(sftp.rename("/Survival-7/hello.txt", "/Survival-7/world/moved.txt").await.is_err());
	let truncate = FileAttributes { size: Some(1), ..FileAttributes::empty() };
	sftp.set_metadata("/Survival-7/world/moved.txt", truncate).await.unwrap();
	assert_eq!(std::fs::read(writable.join("world/moved.txt")).unwrap(), b"d");
	assert!(sftp.remove_file("/Archive-8/hello.txt").await.is_err());
	sftp.remove_file("/Survival-7/world/moved.txt").await.unwrap();
	sftp.remove_dir("/Survival-7/world").await.unwrap();
	assert!(sftp.remove_dir("/Survival-7").await.is_err());

	assert!(sftp.write("/Archive-8/blocked.txt", b"nope").await.is_err());
	assert!(sftp.read("/Survival-7/../../../etc/passwd").await.is_err());
	assert!(sftp.read("/Hidden-9/anything").await.is_err());

	server.stop();
}
