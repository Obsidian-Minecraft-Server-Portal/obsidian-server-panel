use std::collections::HashMap;
use std::path::{Path, PathBuf};

use futures_util::{StreamExt, stream};
use obsidian_common::{CommonError, Result, TaskHandle, UserId};
use serde_json::json;

use super::decisions;
use super::draft::{Draft, ExtraFile};
use super::evidence::{self, JarFacts};
use super::model::{AnalysisResult, AnalyzeRequest, AnalyzedMod, FolderChoice};
use super::score;
use crate::archive;
use crate::http;
use crate::manager::{ContentManager, safe_filename, verify_checksums};
use crate::model::HashMatch;
use crate::modpack::{ListedFile, Listing, Origin, PackEnv, read as read_listing};
use crate::platform::{ContentPlatform, Kind, Platform};

const FILE_CONCURRENCY: usize = 6;
pub(super) const EXCLUDED_FOLDERS: [&str; 5] = ["resourcepacks", "shaderpacks", "screenshots", "saves", "logs"];
const MOD_FOLDERS: [&str; 2] = ["mods", "plugins"];

#[derive(Clone)]
struct Link {
	platform: Platform,
	project_id: String,
	version_id: String,
}

struct Resolved {
	relative: String,
	declared: PackEnv,
	url: Option<String>,
	sha1: Option<String>,
	link: Option<Link>,
}

struct Job {
	id: String,
	dir: PathBuf,
	owner: UserId,
	platform: Platform,
	project_id: String,
}

impl ContentManager {
	pub fn start_analysis(&self, owner: UserId, request: AnalyzeRequest) -> Result<u64> {
		let task = self
			.tasks
			.create("modpack.analyze", format!("Analyzing {}", request.project_id), Some(owner));
		let task_id = task.id();
		let manager = self.clone();
		tokio::spawn(async move {
			match manager.run_analysis(owner, request, &task).await {
				Ok(draft_id) => task.finish(json!({ "draft_id": draft_id })),
				Err(error) => task.fail(error),
			}
		});
		Ok(task_id)
	}

	async fn run_analysis(&self, owner: UserId, request: AnalyzeRequest, task: &TaskHandle) -> Result<String> {
		task.progress(0.02, "Resolving modpack");
		let provider = self.provider(request.platform)?.clone();
		let version = provider.version(&request.project_id, &request.version_id).await?;
		let url = version
			.download_url
			.as_deref()
			.ok_or_else(|| CommonError::Conflict("this modpack must be downloaded manually".into()))?;
		task.progress(0.05, "Downloading modpack");
		let archive_bytes = http::download(&self.http, url).await?;
		let listing = read_listing(request.platform, &archive_bytes)?;
		let id = new_draft_id();
		let job = Job {
			dir: self.paths.cache_dir.join("serverpacks").join(&id),
			id,
			owner,
			platform: request.platform,
			project_id: request.project_id,
		};
		let built = self
			.build_draft(&job, provider.as_ref(), listing, archive_bytes, task)
			.await;
		if built.is_err() {
			let _ = tokio::fs::remove_dir_all(&job.dir).await;
		}
		built.map(|()| job.id)
	}

	async fn build_draft(
		&self,
		job: &Job,
		provider: &dyn ContentPlatform,
		mut listing: Listing,
		archive_bytes: Vec<u8>,
		task: &TaskHandle,
	) -> Result<()> {
		for folder in ["mods", "plugins", "icons"] {
			tokio::fs::create_dir_all(job.dir.join(folder)).await?;
		}
		tokio::fs::write(job.dir.join("pack.zip"), &archive_bytes).await?;
		let mut folders = archive::override_folders(&archive_bytes, &listing.overrides)?;
		task.progress(0.1, "Resolving pack files");
		let files = std::mem::take(&mut listing.files);
		let resolved = self.resolve(provider, files).await?;
		let (jars, others): (Vec<Resolved>, Vec<Resolved>) =
			resolved.into_iter().partition(|file| jar_kind(&file.relative).is_some());
		let mut extras = Vec::new();
		for file in &others {
			if let Some(folder) = top_folder(&file.relative) {
				folders.insert(folder);
			}
			if let Some(url) = &file.url {
				extras.push(ExtraFile {
					relative: file.relative.clone(),
					url: url.clone(),
					sha1: file.sha1.clone(),
				});
			}
		}
		for folder in MOD_FOLDERS {
			folders.remove(folder);
		}
		task.progress(0.2, "Downloading mods");
		let paths = self.download_jars(&jars, &job.dir).await;
		task.progress(0.6, "Inspecting jars");
		let facts = inspect_all(paths.clone()).await?;
		task.progress(0.8, "Matching Modrinth projects");
		let sha1s: Vec<String> = jars
			.iter()
			.filter_map(|jar| jar.sha1.as_deref().map(str::to_ascii_lowercase))
			.collect();
		let matches = self.hash_matches(sha1s).await;
		let remembered = decisions::load(self.db.pool()).await?;
		task.progress(0.9, "Scoring mods");
		let mut mods = Vec::with_capacity(jars.len());
		for (index, (((jar, path), facts), sha1)) in jars
			.iter()
			.zip(&paths)
			.zip(facts)
			.zip(jars.iter().map(|jar| jar.sha1.as_deref().map(str::to_ascii_lowercase)))
			.enumerate()
		{
			let hit = sha1.as_deref().and_then(|key| matches.get(key));
			let mut signals = evidence::pack_signals(jar.declared.client.as_deref(), jar.declared.server.as_deref());
			if let Some(hit) = hit {
				signals.extend(evidence::discover_signals(hit.client_side.as_str(), hit.server_side.as_str()));
			}
			if let Some(facts) = &facts {
				signals.extend(evidence::jar_signals(&facts.detection));
				signals.extend(evidence::class_signals(facts.refs));
			}
			let verdict = score::decide(&signals);
			let mut evidence_lines = verdict.evidence;
			if path.is_none() {
				evidence_lines.push(if jar.url.is_some() {
					"could not be downloaded".into()
				} else {
					"must be downloaded manually".into()
				});
			}
			let name = hit
				.map(|hit| hit.title.clone())
				.or_else(|| facts.as_ref().and_then(|facts| facts.name.clone()))
				.unwrap_or_else(|| file_name(&jar.relative).to_owned());
			let has_icon = match facts.as_ref().and_then(|facts| facts.icon.as_deref()) {
				Some(icon) => {
					tokio::fs::write(job.dir.join("icons").join(format!("{index}.png")), icon).await?;
					true
				}
				None => false,
			};
			let link = jar.link.clone().or_else(|| {
				hit.map(|hit| Link {
					platform: Platform::Modrinth,
					project_id: hit.project_id.clone(),
					version_id: hit.version_id.clone(),
				})
			});
			mods.push(AnalyzedMod {
				index,
				filename: file_name(&jar.relative).to_owned(),
				name,
				kind: jar_kind(&jar.relative).unwrap_or(Kind::Mod),
				side: verdict.side,
				leaning: verdict.leaning,
				confidence: verdict.confidence,
				evidence: evidence_lines,
				downloadable: path.is_some(),
				remembered: sha1.as_ref().and_then(|key| remembered.get(key).copied()),
				has_icon,
				platform: link.as_ref().map(|link| link.platform),
				project_id: link.as_ref().map(|link| link.project_id.clone()),
				version_id: link.as_ref().map(|link| link.version_id.clone()),
				sha1,
			});
		}
		let folders = folders
			.into_iter()
			.map(|name| FolderChoice {
				include: !EXCLUDED_FOLDERS.contains(&name.as_str()),
				name,
			})
			.collect();
		let result = AnalysisResult {
			draft_id: job.id.clone(),
			pack_name: listing.name,
			mc_version: listing.mc_version,
			loader: listing.loader,
			mods,
			folders,
		};
		self.drafts.insert(
			job.id.clone(),
			Draft {
				owner: job.owner,
				dir: job.dir.clone(),
				platform: job.platform,
				project_id: job.project_id.clone(),
				loader_version: listing.loader_version,
				result,
				mod_paths: paths,
				extras,
				overrides: listing.overrides,
			},
		);
		Ok(())
	}

	async fn resolve(&self, provider: &dyn ContentPlatform, files: Vec<ListedFile>) -> Result<Vec<Resolved>> {
		let resolved: Vec<Result<Resolved>> = stream::iter(files)
			.map(|file| self.resolve_file(provider, file))
			.buffered(FILE_CONCURRENCY)
			.collect()
			.await;
		resolved.into_iter().collect()
	}

	async fn resolve_file(&self, provider: &dyn ContentPlatform, file: ListedFile) -> Result<Resolved> {
		match file.origin {
			Origin::Direct { url, sha1 } => Ok(Resolved {
				relative: file.path.unwrap_or_default(),
				declared: file.declared,
				url: Some(url),
				sha1,
				link: None,
			}),
			Origin::CurseForge { project_id, file_id } => {
				let version = provider.version(&project_id, &file_id).await?;
				Ok(Resolved {
					relative: format!("mods/{}", safe_filename(&version.file_name)?),
					declared: file.declared,
					url: version.download_url,
					sha1: version.sha1,
					link: Some(Link {
						platform: Platform::CurseForge,
						project_id,
						version_id: file_id,
					}),
				})
			}
		}
	}

	async fn download_jars(&self, jars: &[Resolved], dir: &Path) -> Vec<Option<PathBuf>> {
		let requests: Vec<(usize, Option<String>, Option<String>, String)> = jars
			.iter()
			.enumerate()
			.map(|(index, jar)| (index, jar.url.clone(), jar.sha1.clone(), jar.relative.clone()))
			.collect();
		let mut downloaded: Vec<(usize, Option<PathBuf>)> = stream::iter(requests)
			.map(|(index, url, sha1, relative)| {
				let dir = dir.to_path_buf();
				async move { (index, self.fetch_jar(url, sha1, relative, dir).await) }
			})
			.buffer_unordered(FILE_CONCURRENCY)
			.collect()
			.await;
		downloaded.sort_by_key(|(index, _)| *index);
		downloaded.into_iter().map(|(_, path)| path).collect()
	}

	async fn fetch_jar(&self, url: Option<String>, sha1: Option<String>, relative: String, dir: PathBuf) -> Option<PathBuf> {
		let bytes = http::download(&self.http, url.as_deref()?).await.ok()?;
		verify_checksums(&bytes, sha1.as_deref(), None).ok()?;
		let folder = relative.split('/').next()?;
		let target = dir.join(folder).join(safe_filename(file_name(&relative)).ok()?);
		tokio::fs::write(&target, &bytes).await.ok()?;
		Some(target)
	}

	async fn hash_matches(&self, sha1s: Vec<String>) -> HashMap<String, HashMatch> {
		if sha1s.is_empty() {
			return HashMap::new();
		}
		let Ok(provider) = self.provider(Platform::Modrinth) else {
			return HashMap::new();
		};
		match provider.lookup_hashes(&sha1s).await {
			Ok(matches) => matches
				.into_iter()
				.map(|hit| (hit.sha1.to_ascii_lowercase(), hit))
				.collect(),
			Err(error) => {
				tracing::warn!(%error, "Modrinth could not match the modpack's mods");
				HashMap::new()
			}
		}
	}
}

async fn inspect_all(paths: Vec<Option<PathBuf>>) -> Result<Vec<Option<JarFacts>>> {
	tokio::task::spawn_blocking(move || {
		paths
			.iter()
			.map(|path| path.as_deref().and_then(evidence::inspect))
			.collect::<Vec<_>>()
	})
	.await
	.map_err(|error| CommonError::Other(error.to_string()))
}

fn jar_kind(relative: &str) -> Option<Kind> {
	let mut parts = relative.split('/');
	let folder = parts.next()?;
	let file = parts.next()?;
	if parts.next().is_some() || !file.to_ascii_lowercase().ends_with(".jar") {
		return None;
	}
	match folder {
		"mods" => Some(Kind::Mod),
		"plugins" => Some(Kind::Plugin),
		_ => None,
	}
}

fn top_folder(relative: &str) -> Option<String> {
	let mut parts = relative.split('/');
	let folder = parts.next()?;
	parts.next().map(|_| folder.to_owned())
}

fn file_name(relative: &str) -> &str {
	relative.rsplit('/').next().unwrap_or(relative)
}

pub(super) fn new_draft_id() -> String {
	hex::encode(rand::random::<[u8; 16]>())
}
