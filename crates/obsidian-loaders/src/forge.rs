use std::collections::HashSet;

use async_trait::async_trait;
use obsidian_common::{CommonError, Loader, Result, TaskHandle};
use tokio::process::Command;

use crate::http::{download, get_text};
use crate::progress::{Stage, run_observed};
use crate::versions::{neoforge_minecraft_version, parse_maven_versions, sort_newest_first, validate_version};
use crate::{InstallRequest, LaunchSpec, ServerLoader, cache, finalize, is_file};

pub const FORGE: Installer = Installer {
	kind: Loader::Forge,
	repo: "https://maven.minecraftforge.net",
	group: "net/minecraftforge/forge",
	artifact: "forge",
	scheme: Scheme::Forge,
};

pub const NEOFORGE: Installer = Installer {
	kind: Loader::NeoForge,
	repo: "https://maven.neoforged.net/releases",
	group: "net/neoforged/neoforge",
	artifact: "neoforge",
	scheme: Scheme::NeoForge,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scheme {
	Forge,
	NeoForge,
}

#[derive(Debug, Clone, Copy)]
pub struct Installer {
	kind: Loader,
	repo: &'static str,
	group: &'static str,
	artifact: &'static str,
	scheme: Scheme,
}

impl Installer {
	fn minecraft_of(self, version: &str) -> Option<String> {
		match self.scheme {
			Scheme::Forge => version.split_once('-').map(|(mc, _)| mc.to_owned()),
			Scheme::NeoForge => neoforge_minecraft_version(version),
		}
	}

	fn installer_url(self, version: &str) -> String {
		format!("{}/{}/{version}/{}-{version}-installer.jar", self.repo, self.group, self.artifact)
	}

	async fn all_versions(self) -> Result<Vec<String>> {
		let metadata = get_text(&format!("{}/{}/maven-metadata.xml", self.repo, self.group)).await?;
		Ok(parse_maven_versions(&metadata))
	}

	async fn resolve_version(self, request: &InstallRequest) -> Result<String> {
		match &request.loader_version {
			Some(version) => {
				validate_version(version)?;
				Ok(version.clone())
			}
			None => {
				let versions = self.loader_versions(&request.mc_version).await?;
				versions
					.iter()
					.find(|version| !version.ends_with("-beta"))
					.or(versions.first())
					.cloned()
					.ok_or_else(|| CommonError::NotFound(format!("{} build for {}", self.kind, request.mc_version)))
			}
		}
	}

	async fn loader_versions_for(self, mc_version: &str) -> Result<Vec<String>> {
		let mut versions: Vec<String> = self
			.all_versions()
			.await?
			.into_iter()
			.filter(|version| self.minecraft_of(version).as_deref() == Some(mc_version))
			.collect();
		sort_newest_first(&mut versions);
		Ok(versions)
	}

	async fn launch_spec(self, request: &InstallRequest, version: &str) -> Result<LaunchSpec> {
		let args_name = if cfg!(windows) { "win_args.txt" } else { "unix_args.txt" };
		let args_path = format!("libraries/{}/{version}/{args_name}", self.group);
		if is_file(&request.instance_dir.join(&args_path)).await {
			return Ok(LaunchSpec::args_file(&args_path));
		}
		for jar in [format!("{}-{version}.jar", self.artifact), format!("forge-{version}-universal.jar")] {
			if is_file(&request.instance_dir.join(&jar)).await {
				return Ok(LaunchSpec::jar(&jar));
			}
		}
		Err(CommonError::Other(format!("{} installer produced no launch configuration", self.kind)))
	}
}

#[async_trait]
impl ServerLoader for Installer {
	fn kind(&self) -> Loader {
		self.kind
	}

	async fn mc_versions(&self) -> Result<Vec<String>> {
		let installer = *self;
		cache::versions(format!("{}:mc", self.kind), async move {
			let mut seen = HashSet::new();
			let mut minecraft: Vec<String> = installer
				.all_versions()
				.await?
				.iter()
				.filter_map(|version| installer.minecraft_of(version))
				.collect();
			minecraft.retain(|mc| seen.insert(mc.clone()));
			sort_newest_first(&mut minecraft);
			Ok(minecraft)
		})
		.await
	}

	async fn loader_versions(&self, mc_version: &str) -> Result<Vec<String>> {
		validate_version(mc_version)?;
		let installer = *self;
		let mc = mc_version.to_owned();
		cache::versions(format!("{}:loader:{mc}", self.kind), async move { installer.loader_versions_for(&mc).await })
			.await
	}

	async fn install(&self, request: &InstallRequest, task: &TaskHandle) -> Result<LaunchSpec> {
		validate_version(&request.mc_version)?;
		let stage = Stage::new(task);
		let version = self.resolve_version(request).await?;

		let installer_jar = request
			.cache_dir
			.join("installers")
			.join(self.artifact)
			.join(format!("{version}-installer.jar"));
		if !is_file(&installer_jar).await {
			download(&self.installer_url(&version), &installer_jar, stage.part(0.0, 0.4), "Downloading installer").await?;
		}

		tokio::fs::create_dir_all(&request.instance_dir).await?;
		let mut command = Command::new(&request.java);
		command.arg("-jar").arg(&installer_jar).arg("--installServer").current_dir(&request.instance_dir);
		run_observed(command, stage.part(0.4, 0.6)).await?;

		let spec = self.launch_spec(request, &version).await?;
		finalize(request, spec).await
	}
}
