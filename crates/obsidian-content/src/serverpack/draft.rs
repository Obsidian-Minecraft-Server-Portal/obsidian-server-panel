use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use moka::sync::Cache;
use obsidian_common::UserId;

use super::model::AnalysisResult;
use crate::platform::Platform;

const DRAFT_TTL: Duration = Duration::from_secs(2 * 60 * 60);
const MAX_DRAFTS: u64 = 32;

pub(crate) struct ExtraFile {
	pub relative: String,
	pub url: String,
	pub sha1: Option<String>,
}

pub(crate) struct Draft {
	pub owner: UserId,
	pub dir: PathBuf,
	pub platform: Platform,
	pub project_id: String,
	pub loader_version: Option<String>,
	pub result: AnalysisResult,
	pub mod_paths: Vec<Option<PathBuf>>,
	pub extras: Vec<ExtraFile>,
	pub overrides: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct DraftStore {
	drafts: Cache<String, Arc<Draft>>,
}

impl DraftStore {
	pub(crate) fn new() -> Self {
		let drafts = Cache::builder()
			.time_to_live(DRAFT_TTL)
			.max_capacity(MAX_DRAFTS)
			.eviction_listener(|_id, draft: Arc<Draft>, _cause| {
				let dir = draft.dir.clone();
				std::thread::spawn(move || {
					let _ = std::fs::remove_dir_all(dir);
				});
			})
			.build();
		Self { drafts }
	}

	pub(crate) fn insert(&self, id: String, draft: Draft) {
		self.drafts.insert(id, Arc::new(draft));
	}

	pub(crate) fn get(&self, id: &str) -> Option<Arc<Draft>> {
		self.drafts.get(id)
	}

	pub(crate) fn remove(&self, id: &str) {
		self.drafts.invalidate(id);
	}
}
