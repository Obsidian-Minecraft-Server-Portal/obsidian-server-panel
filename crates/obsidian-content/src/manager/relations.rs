use std::collections::HashMap;

use obsidian_common::{CommonError, Result};
use serde::Serialize;
use utoipa::ToSchema;

use super::{ContentManager, store};
use crate::metadata::Dependency;
use crate::model::InstalledContent;

#[derive(Debug, Serialize, ToSchema)]
pub struct DependencyEntry {
	pub id: String,
	pub required: bool,
	pub installed: Option<InstalledContent>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DependentEntry {
	pub installed: InstalledContent,
	pub required: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ContentRelations {
	pub dependencies: Vec<DependencyEntry>,
	pub dependents: Vec<DependentEntry>,
}

impl ContentManager {
	pub async fn relations(&self, instance_id: i64, content_id: i64) -> Result<ContentRelations> {
		let pool = self.db.pool();
		let metadata: HashMap<i64, (Option<String>, Vec<Dependency>)> = store::metadata_index(pool, instance_id)
			.await?
			.into_iter()
			.map(|(id, mod_id, depends)| (id, (mod_id, depends)))
			.collect();
		let rows: Vec<(InstalledContent, Option<String>, Vec<Dependency>)> = store::list(pool, instance_id)
			.await?
			.into_iter()
			.map(|item| {
				let (mod_id, depends) = metadata.get(&item.id).cloned().unwrap_or_default();
				(item, mod_id, depends)
			})
			.collect();
		let Some((target, target_mod_id, target_depends)) = rows.iter().find(|(item, _, _)| item.id == content_id) else {
			return Err(CommonError::NotFound("content".into()));
		};

		let installed_by_mod_id: HashMap<String, &InstalledContent> = rows
			.iter()
			.filter_map(|(item, mod_id, _)| mod_id.as_deref().map(|mod_id| (normalize(mod_id), item)))
			.collect();
		let dependencies = target_depends
			.iter()
			.map(|dependency| DependencyEntry {
				id: dependency.id.clone(),
				required: dependency.required,
				installed: installed_by_mod_id.get(&normalize(&dependency.id)).map(|item| (*item).clone()),
			})
			.collect();

		let dependents = match target_mod_id {
			Some(target_mod_id) => rows
				.iter()
				.filter(|(item, _, _)| item.id != target.id)
				.filter_map(|(item, _, depends)| {
					depends
						.iter()
						.find(|dependency| normalize(&dependency.id) == normalize(target_mod_id))
						.map(|dependency| DependentEntry {
							installed: item.clone(),
							required: dependency.required,
						})
				})
				.collect(),
			None => Vec::new(),
		};

		Ok(ContentRelations {
			dependencies,
			dependents,
		})
	}
}

fn normalize(mod_id: &str) -> String {
	mod_id.to_ascii_lowercase()
}
