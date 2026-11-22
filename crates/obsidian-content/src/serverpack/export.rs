use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use obsidian_common::{CommonError, Result, TaskHandle, UserId};
use obsidian_instances::{CreateInstance, InstanceManager};
use serde_json::{Value, json};

use super::analyze::EXCLUDED_FOLDERS;
use super::decisions;
use super::draft::{Draft, ExtraFile};
use super::model::{AnalyzedMod, ExportRequest, FolderChoice, FolderDecision, ModDecision};
use crate::archive;
use crate::http;
use crate::manager::{ContentManager, NewRecord, verify_checksums};
use crate::modpack::ensure_trusted;

impl ContentManager {
	pub fn start_export(
		&self,
		instances: Arc<InstanceManager>,
		owner: UserId,
		id: &str,
		request: ExportRequest,
	) -> Result<u64> {
		let draft = self.owned_draft(owner, id)?;
		let task = self
			.tasks
			.create("serverpack.export", format!("Exporting {}", draft.result.pack_name), Some(owner));
		let task_id = task.id();
		let manager = self.clone();
		let id = id.to_owned();
		tokio::spawn(async move {
			match manager.run_export(&instances, owner, draft, request, &task).await {
				Ok(result) => {
					manager.drafts.remove(&id);
					task.finish(result);
				}
				Err(error) => task.fail(error),
			}
		});
		Ok(task_id)
	}

	async fn run_export(
		&self,
		instances: &InstanceManager,
		owner: UserId,
		draft: Arc<Draft>,
		request: ExportRequest,
		task: &TaskHandle,
	) -> Result<Value> {
		task.progress(0.02, "Creating server");
		let include = include_decisions(&draft.result.mods, &request.mods);
		let folders = folder_decisions(&draft.result.folders, &request.folders);
		let name = match request.name.trim() {
			"" => draft.result.pack_name.clone(),
			name => name.to_owned(),
		};
		let instance = instances
			.create(
				CreateInstance {
					name,
					accept_eula: request.accept_eula,
					loader: draft.result.loader,
					mc_version: draft.result.mc_version.clone(),
					loader_version: draft.loader_version.clone(),
					java_runtime_id: None,
					java_component: None,
					port: None,
					upnp_enabled: None,
				},
				owner,
			)
			.await?;
		let (instance_id, directory) = (instance.id, instance.directory);
		let mut warnings = Vec::new();
		let mut included = 0_usize;
		task.progress(0.2, "Copying mods");
		for (view, keep) in draft.result.mods.iter().zip(&include) {
			if !keep {
				continue;
			}
			match draft.mod_paths.get(view.index).and_then(Option::as_ref) {
				Some(path) => {
					self.copy_mod(instance_id, view, path, &draft.dir).await?;
					included += 1;
				}
				None => warnings.push(format!("{} could not be downloaded, so it was left out", view.filename)),
			}
		}
		task.progress(0.6, "Copying extra files");
		for extra in &draft.extras {
			let folder = extra.relative.split('/').next().unwrap_or_default();
			if !folders.get(folder).copied().unwrap_or(!EXCLUDED_FOLDERS.contains(&folder)) {
				continue;
			}
			if let Err(error) = self.fetch_extra(extra, &directory).await {
				warnings.push(format!("{} was not copied: {error}", extra.relative));
			}
		}
		task.progress(0.8, "Applying overrides");
		let archive_bytes = tokio::fs::read(draft.dir.join("pack.zip")).await?;
		let prefixes = draft.overrides.clone();
		let target = directory.clone();
		let keep_folders = folders.clone();
		tokio::task::spawn_blocking(move || {
			archive::extract_prefixes_where(&archive_bytes, &prefixes, &target, |relative| {
				let folder = relative.split('/').next().unwrap_or_default();
				!relative.contains('/')
					|| keep_folders
						.get(folder)
						.copied()
						.unwrap_or(!EXCLUDED_FOLDERS.contains(&folder))
			})
		})
		.await
		.map_err(|error| CommonError::Other(error.to_string()))??;
		for (view, keep) in draft.result.mods.iter().zip(&include) {
			if let Some(sha1) = view.sha1.as_deref() {
				let explicit = (*keep != view.auto_include()).then_some(*keep);
				decisions::save(self.db.pool(), sha1, explicit).await?;
			}
		}
		let provider = self.provider(draft.platform)?;
		warnings.extend(
			self.apply_branding(instances, instance_id, provider.as_ref(), &draft.project_id)
				.await,
		);
		let excluded = include.iter().filter(|keep| !**keep).count();
		Ok(json!({
			"instance_id": instance_id,
			"included": included,
			"excluded": excluded,
			"warnings": warnings,
		}))
	}

	async fn copy_mod(&self, instance_id: i64, view: &AnalyzedMod, path: &Path, draft_dir: &Path) -> Result<()> {
		let bytes = tokio::fs::read(path).await?;
		let icon = if view.has_icon {
			tokio::fs::read(draft_dir.join("icons").join(format!("{}.png", view.index)))
				.await
				.ok()
		} else {
			None
		};
		let record = NewRecord {
			instance_id,
			filename: &view.filename,
			project_id: view.project_id.as_deref(),
			version_id: view.version_id.as_deref(),
			platform: view.platform,
			kind: view.kind,
			icon,
		};
		self.add_file(&record, &bytes).await?;
		Ok(())
	}

	async fn fetch_extra(&self, extra: &ExtraFile, directory: &Path) -> Result<()> {
		ensure_trusted(&extra.url)?;
		let bytes = http::download(&self.http, &extra.url).await?;
		verify_checksums(&bytes, extra.sha1.as_deref(), None)?;
		let target = safe_join(directory, &extra.relative)?;
		if let Some(parent) = target.parent() {
			tokio::fs::create_dir_all(parent).await?;
		}
		tokio::fs::write(target, bytes).await?;
		Ok(())
	}
}

fn include_decisions(mods: &[AnalyzedMod], decisions: &[ModDecision]) -> Vec<bool> {
	let explicit: HashMap<usize, bool> = decisions
		.iter()
		.map(|decision| (decision.index, decision.include))
		.collect();
	mods.iter()
		.map(|view| explicit.get(&view.index).copied().unwrap_or_else(|| view.auto_include()))
		.collect()
}

fn folder_decisions(folders: &[FolderChoice], decisions: &[FolderDecision]) -> HashMap<String, bool> {
	let explicit: HashMap<&str, bool> = decisions
		.iter()
		.map(|decision| (decision.name.as_str(), decision.include))
		.collect();
	folders
		.iter()
		.map(|folder| {
			let include = explicit.get(folder.name.as_str()).copied().unwrap_or(folder.include);
			(folder.name.clone(), include)
		})
		.collect()
}

fn safe_join(root: &Path, relative: &str) -> Result<PathBuf> {
	let path = Path::new(relative);
	if relative.is_empty() || !path.components().all(|component| matches!(component, Component::Normal(_))) {
		return Err(CommonError::Invalid(format!("unsafe path in modpack: {relative}")));
	}
	Ok(root.join(path))
}
