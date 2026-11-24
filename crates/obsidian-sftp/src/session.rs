use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::FileTimes;
use std::io::{ErrorKind, SeekFrom};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant, UNIX_EPOCH};

use obsidian_auth::Permission;
use obsidian_common::{CommonError, UserId};
use russh_sftp::protocol::{
	Attrs, Data, File, FileAttributes, Handle, Name, OpenFlags, SftpHandle, SftpString, Status, StatusCode,
};
use russh_sftp::server::Handler;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::auth::{SftpAuth, SftpInstance};
use crate::jail;

const MAX_READ_LEN: u32 = 64 * 1024;
const READDIR_BATCH: usize = 128;
const OK_MESSAGE: &str = "Ok";
const LANGUAGE: &str = "en-US";
const LISTING_TTL: Duration = Duration::from_secs(2);

enum Node {
	File { file: fs::File, writable: bool },
	Dir(Vec<File>),
}

enum Location {
	Root,
	Instance { instance: SftpInstance, path: PathBuf },
}

#[derive(Clone)]
struct Listing {
	fetched: Instant,
	instances: Arc<Vec<SftpInstance>>,
}

pub struct SftpSession {
	auth: Arc<dyn SftpAuth>,
	user_id: UserId,
	listing: Mutex<Option<Listing>>,
	nodes: HashMap<u64, Node>,
	next_handle: u64,
}

impl SftpSession {
	pub fn new(auth: Arc<dyn SftpAuth>, user_id: UserId) -> Self {
		Self { auth, user_id, listing: Mutex::new(None), nodes: HashMap::new(), next_handle: 1 }
	}

	async fn instances(&self) -> Arc<Vec<SftpInstance>> {
		let cached = self.listing.lock().unwrap_or_else(PoisonError::into_inner).clone();
		if let Some(listing) = cached.as_ref().filter(|listing| listing.fetched.elapsed() < LISTING_TTL) {
			return Arc::clone(&listing.instances);
		}
		match self.auth.instances(self.user_id).await {
			Some(instances) => {
				let instances: Arc<Vec<SftpInstance>> = Arc::new(
					instances.into_iter().filter(|instance| allows(instance, Permission::FILES_SFTP)).collect(),
				);
				*self.listing.lock().unwrap_or_else(PoisonError::into_inner) =
					Some(Listing { fetched: Instant::now(), instances: Arc::clone(&instances) });
				instances
			}
			None => cached.map(|listing| listing.instances).unwrap_or_default(),
		}
	}


	async fn locate(&self, virtual_path: &str) -> Result<Location, StatusCode> {
		let mut parts = Vec::new();
		for component in Path::new(virtual_path).components() {
			match component {
				Component::Normal(part) => parts.push(part),
				Component::CurDir | Component::RootDir => {}
				_ => return Err(StatusCode::PermissionDenied),
			}
		}
		let Some((first, rest)) = parts.split_first() else {
			return Ok(Location::Root);
		};
		let first = first.to_str().ok_or(StatusCode::NoSuchFile)?;
		let instances = self.instances().await;
		let instance = instances
			.iter()
			.find(|instance| folder_name(instance) == first)
			.ok_or(StatusCode::NoSuchFile)?
			.clone();
		let relative: PathBuf = rest.iter().collect();
		let path = jail::resolve_async(&instance.root, relative)
			.await
			.map_err(|e| status_from_error(&e))?;
		Ok(Location::Instance { instance, path })
	}

	/// The instance root itself is only addressable for reads.
	async fn instance_path(&self, virtual_path: &str, perm: Permission) -> Result<PathBuf, StatusCode> {
		match self.locate(virtual_path).await? {
			Location::Root => Err(StatusCode::PermissionDenied),
			Location::Instance { instance, path } => {
				if !allows(&instance, perm) || (path == instance.root && perm != Permission::FILES_READ) {
					return Err(StatusCode::PermissionDenied);
				}
				Ok(path)
			}
		}
	}

	fn insert(&mut self, node: Node) -> Handle {
		let key = self.next_handle;
		self.next_handle += 1;
		self.nodes.insert(key, node);
		Handle { id: 0, handle: SftpHandle::from(key.to_be_bytes().to_vec()) }
	}

	fn node_key(handle: &SftpHandle) -> Result<u64, StatusCode> {
		<[u8; 8]>::try_from(handle.as_bytes()).map(u64::from_be_bytes).map_err(|_| StatusCode::BadMessage)
	}

	fn node(&mut self, handle: &SftpHandle) -> Result<&mut Node, StatusCode> {
		let key = Self::node_key(handle)?;
		self.nodes.get_mut(&key).ok_or(StatusCode::Failure)
	}
}

impl Handler for SftpSession {
	type Error = StatusCode;

	fn unimplemented(&self) -> Self::Error {
		StatusCode::OpUnsupported
	}

	async fn open(
		&mut self,
		id: u32,
		filename: SftpString,
		pflags: OpenFlags,
		_attrs: FileAttributes,
	) -> Result<Handle, Self::Error> {
		let writes = pflags.intersects(OpenFlags::WRITE | OpenFlags::APPEND | OpenFlags::CREATE | OpenFlags::TRUNCATE);
		let perm = if writes { Permission::FILES_WRITE } else { Permission::FILES_READ };
		let path = self.instance_path(&text(&filename), perm).await?;
		let file = fs::OpenOptions::from(std::fs::OpenOptions::from(pflags))
			.open(&path)
			.await
			.map_err(status_from_io)?;
		let mut handle = self.insert(Node::File { file, writable: writes });
		handle.id = id;
		Ok(handle)
	}

	async fn close(&mut self, id: u32, handle: SftpHandle) -> Result<Status, Self::Error> {
		let key = Self::node_key(&handle)?;
		self.nodes.remove(&key).ok_or(StatusCode::Failure)?;
		Ok(ok_status(id))
	}

	async fn read(&mut self, id: u32, handle: SftpHandle, offset: u64, len: u32) -> Result<Data, Self::Error> {
		let Node::File { file, .. } = self.node(&handle)? else {
			return Err(StatusCode::Failure);
		};
		file.seek(SeekFrom::Start(offset)).await.map_err(status_from_io)?;
		let mut data = vec![0; len.min(MAX_READ_LEN) as usize];
		let read = file.read(&mut data).await.map_err(status_from_io)?;
		if read == 0 {
			return Err(StatusCode::Eof);
		}
		data.truncate(read);
		Ok(Data { id, data })
	}

	async fn write(&mut self, id: u32, handle: SftpHandle, offset: u64, data: Vec<u8>) -> Result<Status, Self::Error> {
		let Node::File { file, .. } = self.node(&handle)? else {
			return Err(StatusCode::Failure);
		};
		file.seek(SeekFrom::Start(offset)).await.map_err(status_from_io)?;
		file.write_all(&data).await.map_err(status_from_io)?;
		file.flush().await.map_err(status_from_io)?;
		Ok(ok_status(id))
	}

	async fn fstat(&mut self, id: u32, handle: SftpHandle) -> Result<Attrs, Self::Error> {
		let Node::File { file, .. } = self.node(&handle)? else {
			return Err(StatusCode::Failure);
		};
		let metadata = file.metadata().await.map_err(status_from_io)?;
		Ok(Attrs { id, attrs: FileAttributes::from(&metadata) })
	}

	async fn fsetstat(&mut self, id: u32, handle: SftpHandle, attrs: FileAttributes) -> Result<Status, Self::Error> {
		if !changes_content(&attrs) {
			return Ok(ok_status(id));
		}
		let Node::File { file, writable: true } = self.node(&handle)? else {
			return Err(StatusCode::PermissionDenied);
		};
		let file = file.try_clone().await.map_err(status_from_io)?.into_std().await;
		apply_attrs(file, attrs).await?;
		Ok(ok_status(id))
	}

	async fn setstat(&mut self, id: u32, path: SftpString, attrs: FileAttributes) -> Result<Status, Self::Error> {
		if !changes_content(&attrs) {
			return Ok(ok_status(id));
		}
		let path = self.instance_path(&text(&path), Permission::FILES_WRITE).await?;
		if fs::metadata(&path).await.map_err(status_from_io)?.is_dir() {
			return Ok(ok_status(id));
		}
		let file = fs::OpenOptions::new().write(true).open(&path).await.map_err(status_from_io)?;
		apply_attrs(file.into_std().await, attrs).await?;
		Ok(ok_status(id))
	}

	async fn lstat(&mut self, id: u32, path: SftpString) -> Result<Attrs, Self::Error> {
		self.stat_path(id, &text(&path), false).await
	}

	async fn stat(&mut self, id: u32, path: SftpString) -> Result<Attrs, Self::Error> {
		self.stat_path(id, &text(&path), true).await
	}

	async fn opendir(&mut self, id: u32, path: SftpString) -> Result<Handle, Self::Error> {
		let entries = match self.locate(&text(&path)).await? {
			Location::Root => {
				let instances = self.instances().await;
				instances
					.iter()
					.filter(|instance| allows(instance, Permission::FILES_READ))
					.map(|instance| File::new(folder_name(instance), FileAttributes::dummy()))
					.collect()
			}
			Location::Instance { instance, path } => {
				if !allows(&instance, Permission::FILES_READ) {
					return Err(StatusCode::PermissionDenied);
				}
				read_entries(&path).await?
			}
		};
		let mut handle = self.insert(Node::Dir(entries));
		handle.id = id;
		Ok(handle)
	}

	async fn readdir(&mut self, id: u32, handle: SftpHandle) -> Result<Name, Self::Error> {
		let Node::Dir(entries) = self.node(&handle)? else {
			return Err(StatusCode::Failure);
		};
		if entries.is_empty() {
			return Err(StatusCode::Eof);
		}
		let batch = entries.split_off(entries.len().saturating_sub(READDIR_BATCH));
		Ok(Name { id, files: batch })
	}

	async fn remove(&mut self, id: u32, filename: SftpString) -> Result<Status, Self::Error> {
		let path = self.instance_path(&text(&filename), Permission::FILES_DELETE).await?;
		fs::remove_file(&path).await.map_err(status_from_io)?;
		Ok(ok_status(id))
	}

	async fn mkdir(&mut self, id: u32, path: SftpString, _attrs: FileAttributes) -> Result<Status, Self::Error> {
		let path = self.instance_path(&text(&path), Permission::FILES_WRITE).await?;
		fs::create_dir(&path).await.map_err(status_from_io)?;
		Ok(ok_status(id))
	}

	async fn rmdir(&mut self, id: u32, path: SftpString) -> Result<Status, Self::Error> {
		let path = self.instance_path(&text(&path), Permission::FILES_DELETE).await?;
		fs::remove_dir(&path).await.map_err(status_from_io)?;
		Ok(ok_status(id))
	}

	async fn rename(&mut self, id: u32, oldpath: SftpString, newpath: SftpString) -> Result<Status, Self::Error> {
		let from = self.instance_path(&text(&oldpath), Permission::FILES_WRITE).await?;
		let to = self.instance_path(&text(&newpath), Permission::FILES_WRITE).await?;
		if fs::try_exists(&to).await.map_err(status_from_io)? {
			return Err(StatusCode::Failure);
		}
		fs::rename(&from, &to).await.map_err(status_from_io)?;
		Ok(ok_status(id))
	}

	async fn realpath(&mut self, id: u32, path: SftpString) -> Result<Name, Self::Error> {
		let normalized = normalize_virtual(&text(&path));
		Ok(Name { id, files: vec![File::dummy(normalized)] })
	}
}

impl SftpSession {
	async fn stat_path(&self, id: u32, virtual_path: &str, follow: bool) -> Result<Attrs, StatusCode> {
		let attrs = match self.locate(virtual_path).await? {
			Location::Root => FileAttributes::dummy(),
			Location::Instance { instance, path } => {
				if !allows(&instance, Permission::FILES_READ) {
					return Err(StatusCode::PermissionDenied);
				}
				let metadata = if follow {
					fs::metadata(&path).await
				} else {
					fs::symlink_metadata(&path).await
				};
				FileAttributes::from(&metadata.map_err(status_from_io)?)
			}
		};
		Ok(Attrs { id, attrs })
	}
}

fn changes_content(attrs: &FileAttributes) -> bool {
	attrs.size.is_some() || attrs.mtime.is_some()
}

async fn apply_attrs(file: std::fs::File, attrs: FileAttributes) -> Result<(), StatusCode> {
	let task = move || {
		if let Some(size) = attrs.size {
			file.set_len(size)?;
		}
		if let Some(mtime) = attrs.mtime {
			let at = |secs: u32| UNIX_EPOCH + Duration::from_secs(secs.into());
			let times = FileTimes::new().set_modified(at(mtime)).set_accessed(at(attrs.atime.unwrap_or(mtime)));
			file.set_times(times)?;
		}
		Ok(())
	};
	tokio::task::spawn_blocking(task).await.map_err(|_| StatusCode::Failure)?.map_err(status_from_io)
}

async fn read_entries(dir: &Path) -> Result<Vec<File>, StatusCode> {
	let mut reader = fs::read_dir(dir).await.map_err(status_from_io)?;
	let mut entries = Vec::new();
	while let Some(entry) = reader.next_entry().await.map_err(status_from_io)? {
		let name = entry.file_name().to_string_lossy().into_owned();
		let attrs = match entry.metadata().await {
			Ok(metadata) => FileAttributes::from(&metadata),
			Err(_) => continue,
		};
		entries.push(File::new(name, attrs));
	}
	Ok(entries)
}

fn allows(instance: &SftpInstance, perm: Permission) -> bool {
	Permission::from_bits_truncate(instance.perms).contains(perm)
}

/// Folder name shown in the SFTP root: the instance name with `/` and `\` replaced, plus `-<id>`.
pub fn folder_name(instance: &SftpInstance) -> String {
	let name: String = instance
		.name
		.chars()
		.map(|c| if c == '/' || c == '\\' || c.is_control() { '_' } else { c })
		.collect();
	format!("{name}-{}", instance.id)
}

fn text(path: &SftpString) -> Cow<'_, str> {
	String::from_utf8_lossy(path.as_ref())
}

fn normalize_virtual(path: &str) -> String {
	let mut parts: Vec<&str> = Vec::new();
	for component in Path::new(path).components() {
		match component {
			Component::Normal(part) => parts.push(part.to_str().unwrap_or_default()),
			Component::ParentDir => {
				parts.pop();
			}
			_ => {}
		}
	}
	format!("/{}", parts.join("/"))
}

fn ok_status(id: u32) -> Status {
	Status { id, status_code: StatusCode::Ok, error_message: OK_MESSAGE.into(), language_tag: LANGUAGE.into() }
}

fn status_from_io(error: std::io::Error) -> StatusCode {
	match error.kind() {
		ErrorKind::NotFound => StatusCode::NoSuchFile,
		ErrorKind::PermissionDenied => StatusCode::PermissionDenied,
		_ => StatusCode::Failure,
	}
}

fn status_from_error(error: &CommonError) -> StatusCode {
	match error {
		CommonError::Forbidden(_) => StatusCode::PermissionDenied,
		CommonError::NotFound(_) => StatusCode::NoSuchFile,
		_ => StatusCode::Failure,
	}
}
