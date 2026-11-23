use obsidian_common::{CommonError, Result};

use super::{ContentManager, store};
use crate::model::InstalledContent;
use crate::platform::Platform;

impl ContentManager {
	pub async fn associate(
		&self,
		instance_id: i64,
		content_id: i64,
		platform: Platform,
		project_id: &str,
		version_id: &str,
	) -> Result<InstalledContent> {
		let pool = self.db.pool();
		self.get(instance_id, content_id).await?;
		let version = self.provider(platform)?.version(project_id, version_id).await?;
		if version.project_id != project_id {
			return Err(CommonError::Invalid(format!(
				"version {version_id} does not belong to project {project_id}"
			)));
		}
		let installed = store::list(pool, instance_id).await?;
		if is_linked_elsewhere(&installed, content_id, platform, project_id) {
			return Err(CommonError::Conflict(
				"another file on this instance is already linked to this project".into(),
			));
		}
		store::set_association(pool, content_id, platform, project_id, version_id).await?;
		self.get(instance_id, content_id).await
	}
}

fn is_linked_elsewhere(items: &[InstalledContent], content_id: i64, platform: Platform, project_id: &str) -> bool {
	items.iter().any(|item| {
		item.id != content_id && item.platform == Some(platform) && item.project_id.as_deref() == Some(project_id)
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::platform::Kind;

	fn item(id: i64, platform: Option<Platform>, project_id: Option<&str>) -> InstalledContent {
		InstalledContent {
			id,
			filename: format!("{id}.jar"),
			name: None,
			project_id: project_id.map(str::to_owned),
			version_id: None,
			platform,
			kind: Kind::Mod,
			enabled: true,
			instance_id: 1,
		}
	}

	#[test]
	fn a_project_links_to_one_file_per_platform() {
		let items = [
			item(1, Some(Platform::Modrinth), Some("abc")),
			item(2, None, None),
			item(3, Some(Platform::CurseForge), Some("abc")),
		];
		assert!(!is_linked_elsewhere(&items, 1, Platform::Modrinth, "abc"));
		assert!(is_linked_elsewhere(&items, 2, Platform::Modrinth, "abc"));
		assert!(!is_linked_elsewhere(&items, 2, Platform::Hangar, "abc"));
	}
}
