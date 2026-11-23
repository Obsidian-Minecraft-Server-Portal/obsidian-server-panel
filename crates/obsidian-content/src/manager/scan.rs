use std::collections::HashSet;
use std::path::{Path, PathBuf};

use futures_util::{StreamExt, stream};
use obsidian_common::Result;
use tokio::fs;

use super::store::{self, DISABLED_SUFFIX};
use super::{ContentManager, content_filename, kind_for_loader, subdirectory};
use crate::platform::Kind;

const ICON_CONCURRENCY: usize = 4;

type Scanned = (Kind, String, Option<Vec<u8>>, Option<String>);

struct Folder {
	kind: Kind,
	dir: PathBuf,
	names: HashSet<String>,
}

impl ContentManager {
	pub(crate) async fn refresh(&self, instance_id: i64) -> Result<()> {
		if self.scanned.contains_key(&instance_id) {
			return Ok(());
		}
		let _files = self.files.lock().await;
		if self.scanned.contains_key(&instance_id) {
			return Ok(());
		}
		self.sync_disk(instance_id).await?;
		self.scanned.insert(instance_id, ());
		Ok(())
	}

	async fn sync_disk(&self, instance_id: i64) -> Result<()> {
		let pool = self.db.pool();
		let (loader, _) = store::instance(pool, instance_id).await?;
		let rows = store::list(pool, instance_id).await?;
		for row in rows.iter().filter(|row| row.name.is_none()) {
			let path = self.file_path(instance_id, row.kind, &row.filename).await?;
			let (_, title) = self.extract_metadata(&path).await;
			store::set_name(pool, row.id, title.as_deref().unwrap_or_default()).await?;
		}
		let mut folders = Vec::with_capacity(2);
		for kind in [kind_for_loader(loader), Kind::DataPack] {
			let dir = self.content_dir(instance_id, kind).await?;
			let names = content_files(&dir, kind).await?;
			folders.push(Folder { kind, dir, names });
		}

		let mut missing = Vec::new();
		for row in rows {
			let folder = folders
				.iter_mut()
				.find(|folder| subdirectory(folder.kind) == subdirectory(row.kind));
			let present = match folder {
				Some(folder) => folder.names.remove(&row.filename),
				None => fs::try_exists(self.file_path(instance_id, row.kind, &row.filename).await?).await?,
			};
			if !present {
				missing.push(row);
			}
		}
		if missing.is_empty() && folders.iter().all(|folder| folder.names.is_empty()) {
			return Ok(());
		}

		let fresh = folders
			.iter()
			.flat_map(|folder| folder.names.iter().map(move |name| (folder.kind, folder.dir.join(name), name.clone())));
		let found: Vec<Scanned> = stream::iter(fresh)
			.map(|(kind, path, filename)| async move {
				let (icon, title) = self.extract_metadata(&path).await;
				(kind, filename, icon, title)
			})
			.buffer_unordered(ICON_CONCURRENCY)
			.collect()
			.await;

		let mut stale = Vec::with_capacity(missing.len());
		for row in missing {
			if !fs::try_exists(self.file_path(instance_id, row.kind, &row.filename).await?).await? {
				stale.push(row.id);
			}
		}

		let mut tx = pool.begin().await?;
		for id in stale {
			store::delete(&mut *tx, id).await?;
		}
		for (kind, filename, icon, title) in &found {
			store::insert_untracked(
				&mut *tx,
				instance_id,
				filename,
				*kind,
				icon.as_deref(),
				title.as_deref().unwrap_or_default(),
			)
			.await?;
		}
		tx.commit().await?;
		Ok(())
	}
}

async fn content_files(dir: &Path, kind: Kind) -> Result<HashSet<String>> {
	let mut entries = match fs::read_dir(dir).await {
		Ok(entries) => entries,
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(HashSet::new()),
		Err(error) => return Err(error.into()),
	};
	let mut names = HashSet::new();
	while let Some(entry) = entries.next_entry().await? {
		let Ok(name) = entry.file_name().into_string() else {
			continue;
		};
		if is_content_file(kind, &name) && entry.file_type().await?.is_file() {
			names.insert(name);
		}
	}
	Ok(names)
}

fn is_content_file(kind: Kind, name: &str) -> bool {
	let stem = name.strip_suffix(DISABLED_SUFFIX).unwrap_or(name);
	content_filename(kind, stem).is_ok_and(|base| base == stem)
}
