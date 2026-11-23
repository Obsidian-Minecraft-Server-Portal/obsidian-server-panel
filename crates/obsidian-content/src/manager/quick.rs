use std::collections::HashSet;

use obsidian_common::{CommonError, Result, TaskHandle, UserId};

use super::{ContentManager, store};
use crate::model::{DependencyKind, InstalledContent, ProjectVersion, SideSupport, VersionFilter};
use crate::platform::{ContentPlatform, Kind, Platform};

const MAX_DEPENDENCY_DEPTH: usize = 3;

struct Planned {
	project_id: String,
	version: ProjectVersion,
}

impl ContentManager {
	pub async fn quick_add(
		&self,
		instance_id: i64,
		platform: Platform,
		project_id: &str,
		version_id: Option<&str>,
		owner: Option<UserId>,
	) -> Result<Vec<InstalledContent>> {
		let provider = self.provider(platform)?.clone();
		let project = provider.project(project_id).await?;
		let project_id = project.id.as_str();
		let (loader, mc_version) = store::instance(self.db.pool(), instance_id).await?;
		let filter = VersionFilter {
			mc_version: Some(mc_version.clone()),
			loader: Some(loader),
			kind: Some(Kind::Mod),
		};
		let installed: HashSet<String> = store::list(self.db.pool(), instance_id)
			.await?
			.into_iter()
			.filter(|item| item.platform == Some(platform))
			.filter_map(|item| item.project_id)
			.collect();
		let main = match version_id {
			Some(version) => provider.version(project_id, version).await?,
			None => newest(provider.versions(project_id, &filter).await?).ok_or_else(|| {
				CommonError::NotFound(format!("no version of {project_id} is available for Minecraft {mc_version}"))
			})?,
		};
		let plan = plan(provider.as_ref(), &filter, &installed, project_id, main).await?;

		let task = self.tasks.create("content.quick_add", format!("Adding {project_id}"), owner);
		let outcome = self.install_plan(instance_id, provider.as_ref(), &plan, &task).await;
		match &outcome {
			Ok(installed) => task.finish(serde_json::to_value(installed).unwrap_or_default()),
			Err(error) => task.fail(error),
		}
		outcome
	}

	async fn install_plan(
		&self,
		instance_id: i64,
		provider: &dyn ContentPlatform,
		plan: &[Planned],
		task: &TaskHandle,
	) -> Result<Vec<InstalledContent>> {
		let mut added = Vec::with_capacity(plan.len());
		for step in plan {
			let version_id = step.version.id.clone();
			added.push(
				self.install_version(instance_id, provider, &step.project_id, &version_id, Kind::Mod, task)
					.await?,
			);
		}
		Ok(added)
	}
}

async fn plan(
	provider: &dyn ContentPlatform,
	filter: &VersionFilter,
	installed: &HashSet<String>,
	project_id: &str,
	main: ProjectVersion,
) -> Result<Vec<Planned>> {
	if installed.contains(project_id) {
		return Err(CommonError::Conflict(
			"this project is already installed on the instance; update it from the Content tab instead".into(),
		));
	}
	let mut seen = HashSet::from([project_id.to_owned()]);
	let mut planned = Vec::new();
	let mut pending = vec![(main.clone(), 0usize)];
	while let Some((version, depth)) = pending.pop() {
		for dependency in version
			.dependencies
			.iter()
			.filter(|dependency| dependency.kind == DependencyKind::Required)
		{
			let Some(dependency_project) = dependency.project_id.clone() else {
				continue;
			};
			if installed.contains(&dependency_project) || !seen.insert(dependency_project.clone()) {
				continue;
			}
			if provider.project(&dependency_project).await?.server_side == Some(SideSupport::Unsupported) {
				continue;
			}
			if depth >= MAX_DEPENDENCY_DEPTH {
				return Err(CommonError::Conflict(format!("the dependencies of {project_id} nest too deeply")));
			}
			let resolved = newest(provider.versions(&dependency_project, filter).await?).ok_or_else(|| {
				CommonError::Conflict(format!("required dependency {dependency_project} has no version for this server"))
			})?;
			pending.push((resolved.clone(), depth + 1));
			planned.push(Planned {
				project_id: dependency_project,
				version: resolved,
			});
		}
	}
	planned.reverse();
	planned.push(Planned {
		project_id: project_id.to_owned(),
		version: main,
	});
	Ok(planned)
}

fn newest(versions: Vec<ProjectVersion>) -> Option<ProjectVersion> {
	versions.into_iter().max_by_key(|version| version.published_at)
}

#[cfg(test)]
mod tests {
	use std::collections::HashMap;

	use async_trait::async_trait;
	use chrono::{TimeZone, Utc};

	use super::*;
	use crate::model::{Project, SearchPage, SearchQuery, SideSupport, VersionDependency};

	struct Fake {
		projects: HashMap<String, Project>,
		versions: HashMap<String, Vec<ProjectVersion>>,
	}

	#[async_trait]
	impl ContentPlatform for Fake {
		fn id(&self) -> Platform {
			Platform::Modrinth
		}

		fn supports(&self, _kind: Kind) -> bool {
			true
		}

		async fn search(&self, _query: &SearchQuery) -> Result<SearchPage> {
			Err(CommonError::Invalid("unused".into()))
		}

		async fn project(&self, id: &str) -> Result<Project> {
			self.projects.get(id).cloned().ok_or_else(|| CommonError::NotFound(id.into()))
		}

		async fn versions(&self, id: &str, _filter: &VersionFilter) -> Result<Vec<ProjectVersion>> {
			Ok(self.versions.get(id).cloned().unwrap_or_default())
		}

		async fn version(&self, _project_id: &str, _version_id: &str) -> Result<ProjectVersion> {
			Err(CommonError::Invalid("unused".into()))
		}
	}

	fn project(id: &str, server_side: Option<SideSupport>) -> Project {
		Project {
			platform: Platform::Modrinth,
			id: id.to_owned(),
			slug: id.to_owned(),
			name: id.to_owned(),
			summary: String::new(),
			icon_url: None,
			banner_url: None,
			downloads: 0,
			kind: Kind::Mod,
			client_side: None,
			server_side,
		}
	}

	fn version(project_id: &str, dependencies: &[(&str, DependencyKind)]) -> ProjectVersion {
		ProjectVersion {
			id: format!("{project_id}-v1"),
			project_id: project_id.to_owned(),
			name: project_id.to_owned(),
			version_number: "1.0.0".to_owned(),
			mc_versions: vec!["1.21.4".to_owned()],
			loaders: vec!["fabric".to_owned()],
			file_name: format!("{project_id}.jar"),
			download_url: None,
			sha1: None,
			sha256: None,
			published_at: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap(),
			has_server_pack: false,
			channel: "release".to_owned(),
			downloads: 0,
			dependencies: dependencies
				.iter()
				.map(|(id, kind)| VersionDependency {
					project_id: Some((*id).to_owned()),
					version_id: None,
					kind: *kind,
				})
				.collect(),
		}
	}

	fn fake() -> Fake {
		let projects = [
			project("app", Some(SideSupport::Required)),
			project("lib", Some(SideSupport::Required)),
			project("core", Some(SideSupport::Required)),
			project("shader", Some(SideSupport::Unsupported)),
		]
		.into_iter()
		.map(|item| (item.id.clone(), item))
		.collect();
		let versions = [
			("app", version("app", &[("lib", DependencyKind::Required), ("shader", DependencyKind::Required)])),
			("lib", version("lib", &[("core", DependencyKind::Required), ("app", DependencyKind::Optional)])),
			("core", version("core", &[])),
		]
		.into_iter()
		.map(|(id, item)| (id.to_owned(), vec![item]))
		.collect();
		Fake { projects, versions }
	}

	fn filter() -> VersionFilter {
		VersionFilter {
			mc_version: Some("1.21.4".to_owned()),
			loader: Some(obsidian_common::Loader::Fabric),
			kind: Some(Kind::Mod),
		}
	}

	fn run(installed: &[&str]) -> Result<Vec<String>> {
		let fake = fake();
		let installed: HashSet<String> = installed.iter().map(|id| (*id).to_owned()).collect();
		let main = fake.versions["app"][0].clone();
		let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
		let planned = runtime.block_on(plan(&fake, &filter(), &installed, "app", main))?;
		Ok(planned.into_iter().map(|step| step.project_id).collect())
	}

	#[test]
	fn plan_installs_required_dependencies_first_and_skips_the_rest() {
		assert_eq!(run(&[]).unwrap(), ["core", "lib", "app"]);
		assert_eq!(run(&["core"]).unwrap(), ["lib", "app"]);
	}

	#[test]
	fn plan_refuses_a_project_that_is_already_installed() {
		assert!(matches!(run(&["app"]), Err(CommonError::Conflict(_))));
	}

	#[test]
	fn plan_fails_when_a_required_dependency_has_no_version() {
		let mut fake = fake();
		fake.versions.remove("core");
		let main = fake.versions["app"][0].clone();
		let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
		let outcome = runtime.block_on(plan(&fake, &filter(), &HashSet::new(), "app", main));
		assert!(matches!(outcome, Err(CommonError::Conflict(_))));
	}
}
