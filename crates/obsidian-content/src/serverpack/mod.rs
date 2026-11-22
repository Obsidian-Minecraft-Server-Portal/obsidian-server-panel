mod analyze;
mod classes;
mod decisions;
mod draft;
mod evidence;
mod export;
mod model;
mod score;
#[cfg(test)]
mod tests;

use obsidian_common::{CommonError, Result, UserId};

pub(crate) use draft::DraftStore;
use draft::Draft;
pub use model::{
	AnalysisResult, AnalyzeRequest, AnalyzedMod, DetectedSide, ExportRequest, FolderChoice, FolderDecision, ModDecision,
};

use crate::manager::ContentManager;

impl ContentManager {
	pub(super) fn owned_draft(&self, owner: UserId, id: &str) -> Result<std::sync::Arc<Draft>> {
		self.drafts
			.get(id)
			.filter(|draft| draft.owner == owner)
			.ok_or_else(|| CommonError::NotFound(format!("server pack {id}")))
	}

	pub fn draft(&self, owner: UserId, id: &str) -> Result<AnalysisResult> {
		Ok(self.owned_draft(owner, id)?.result.clone())
	}

	pub async fn draft_icon(&self, owner: UserId, id: &str, index: usize) -> Result<Vec<u8>> {
		let draft = self.owned_draft(owner, id)?;
		tokio::fs::read(draft.dir.join("icons").join(format!("{index}.png")))
			.await
			.map_err(|_| CommonError::NotFound(format!("icon for mod {index}")))
	}

	pub fn discard_draft(&self, owner: UserId, id: &str) -> Result<()> {
		self.owned_draft(owner, id)?;
		self.drafts.remove(id);
		Ok(())
	}
}
