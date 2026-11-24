use std::collections::HashSet;

use obsidian_common::Loader;

use super::Candidate;
use crate::manager::ContentManager;
use crate::model::{Project, ProjectVersion, SearchQuery, VersionFilter};
use crate::platform::{Kind, Platform};

const SEARCH_LIMIT: u32 = 5;
const LOADER_WORDS: [&str; 10] = ["fabric", "forge", "neoforge", "quilt", "paper", "purpur", "spigot", "bukkit", "mc", "mod"];

impl ContentManager {
	pub(super) async fn search_match(
		&self,
		platform: Platform,
		candidate: &Candidate,
		loader: Loader,
	) -> Option<(String, String)> {
		let kind = candidate.row.kind();
		let loader = (kind != Kind::DataPack).then_some(loader);
		let filter = VersionFilter {
			mc_version: None,
			loader,
			kind: Some(kind),
		};
		let mut checked = HashSet::new();
		let terms = query_terms(candidate.row.name.as_deref(), candidate.row.mod_id.as_deref(), &candidate.stem);
		for term in terms {
			let query = search_query(&term, kind, loader);
			let page = match self.search(platform, &query).await {
				Ok(page) => page,
				Err(error) => {
					tracing::debug!(%error, platform = platform.name(), term, "content search failed while matching");
					return None;
				}
			};
			let wanted = normalize(&term);
			for project in page.items.iter().filter(|project| names_match(project, &wanted)) {
				if !checked.insert(project.id.clone()) {
					continue;
				}
				let Ok(versions) = self.versions(platform, &project.id, &filter).await else {
					continue;
				};
				if let Some(version) = pick_version(&versions, &candidate.filename, &candidate.sha1) {
					return Some((project.id.clone(), version.id.clone()));
				}
			}
		}
		None
	}
}

fn search_query(text: &str, kind: Kind, loader: Option<Loader>) -> SearchQuery {
	SearchQuery {
		text: text.to_owned(),
		kind,
		mc_version: None,
		loader,
		categories: Vec::new(),
		excluded_categories: Vec::new(),
		excluded_loaders: Vec::new(),
		environments: Vec::new(),
		open_source: false,
		offset: 0,
		limit: SEARCH_LIMIT,
	}
}

pub(super) fn query_terms(name: Option<&str>, mod_id: Option<&str>, stem: &str) -> Vec<String> {
	let from_file = filename_name(stem);
	let mut seen = HashSet::new();
	[name.map(str::to_owned), mod_id.map(str::to_owned), from_file]
		.into_iter()
		.flatten()
		.map(|term| term.trim().to_owned())
		.filter(|term| normalize(term).len() >= 2 && seen.insert(normalize(term)))
		.collect()
}

pub(super) fn filename_name(stem: &str) -> Option<String> {
	let mut kept: Vec<&str> = Vec::new();
	for token in stem.split(['-', '_', '+', ' ']).filter(|token| !token.is_empty()) {
		let lower = token.to_ascii_lowercase();
		if is_version_token(&lower) || (!kept.is_empty() && LOADER_WORDS.contains(&lower.as_str())) {
			break;
		}
		kept.push(token);
	}
	Some(kept.join(" ")).filter(|name| !name.is_empty())
}

fn is_version_token(token: &str) -> bool {
	let rest = token
		.strip_prefix("mc")
		.or_else(|| token.strip_prefix('v'))
		.unwrap_or(token);
	rest.starts_with(|c: char| c.is_ascii_digit())
}

pub(super) fn normalize(value: &str) -> String {
	value
		.chars()
		.filter(char::is_ascii_alphanumeric)
		.map(|c| c.to_ascii_lowercase())
		.collect()
}

fn names_match(project: &Project, wanted: &str) -> bool {
	normalize(&project.slug) == wanted || normalize(&project.name) == wanted
}

pub(super) fn pick_version<'a>(versions: &'a [ProjectVersion], filename: &str, sha1: &str) -> Option<&'a ProjectVersion> {
	let exact = versions.iter().find(|version| {
		version.file_name.eq_ignore_ascii_case(filename)
			|| version
				.sha1
				.as_deref()
				.is_some_and(|hash| hash.eq_ignore_ascii_case(sha1))
	});
	if exact.is_some() {
		return exact;
	}
	let stem = file_stem(filename).to_ascii_lowercase();
	versions
		.iter()
		.filter(|version| contains_token(&stem, &version.version_number.to_ascii_lowercase()))
		.max_by_key(|version| version.version_number.len())
}

pub(super) fn file_stem(filename: &str) -> &str {
	filename
		.rsplit_once('.')
		.map_or(filename, |(stem, _)| stem)
}

fn contains_token(haystack: &str, needle: &str) -> bool {
	if needle.len() < 2 || !needle.contains(|c: char| c.is_ascii_digit()) {
		return false;
	}
	let is_boundary = |c: char| !(c.is_ascii_alphanumeric() || c == '.');
	haystack.match_indices(needle).any(|(start, _)| {
		let before = haystack[..start].chars().next_back().is_none_or(is_boundary);
		let after = haystack[start + needle.len()..].chars().next().is_none_or(is_boundary);
		before && after
	})
}

#[cfg(test)]
mod tests {
	use chrono::Utc;

	use super::*;

	fn version(id: &str, number: &str, file_name: &str, sha1: Option<&str>) -> ProjectVersion {
		ProjectVersion {
			id: id.to_owned(),
			project_id: "p".to_owned(),
			name: number.to_owned(),
			version_number: number.to_owned(),
			mc_versions: Vec::new(),
			loaders: Vec::new(),
			file_name: file_name.to_owned(),
			download_url: None,
			sha1: sha1.map(str::to_owned),
			sha256: None,
			published_at: Utc::now(),
			has_server_pack: false,
			channel: "release".to_owned(),
			downloads: 0,
			dependencies: Vec::new(),
		}
	}

	#[test]
	fn filename_stems_drop_versions_and_loader_tags() {
		assert_eq!(filename_name("sodium-fabric-0.5.8+mc1.20.4").as_deref(), Some("sodium"));
		assert_eq!(filename_name("jei-1.20.1-forge-15.2.0.27").as_deref(), Some("jei"));
		assert_eq!(filename_name("fabric-api-0.92.0+1.20.1").as_deref(), Some("fabric api"));
		assert_eq!(filename_name("Xaeros_Minimap_24.0.0_Fabric_1.20.4").as_deref(), Some("Xaeros Minimap"));
		assert_eq!(filename_name("EssentialsX-2.20.1").as_deref(), Some("EssentialsX"));
		assert_eq!(filename_name("worldedit-bukkit-7.3.0").as_deref(), Some("worldedit"));
		assert_eq!(filename_name("lithium-mc1.21-0.13.0").as_deref(), Some("lithium"));
		assert_eq!(filename_name("1.20.1"), None);
	}

	#[test]
	fn terms_are_deduplicated_by_normalized_form() {
		let terms = query_terms(Some("Sodium"), Some("sodium"), "sodium-fabric-0.5.8");
		assert_eq!(terms, ["Sodium"]);
		let terms = query_terms(Some("Just Enough Items"), Some("jei"), "jei-1.20.1-forge-15.2.0.27");
		assert_eq!(terms, ["Just Enough Items", "jei"]);
	}

	#[test]
	fn normalize_keeps_lowercase_alphanumerics() {
		assert_eq!(normalize("Fabric API"), "fabricapi");
		assert_eq!(normalize("fabric-api"), "fabricapi");
	}

	#[test]
	fn versions_match_by_filename_then_hash_then_version_token() {
		let versions = [
			version("a", "0.5.7", "sodium-fabric-0.5.7.jar", None),
			version("b", "0.5.8", "sodium-fabric-0.5.8+mc1.20.4.jar", Some("ABC")),
		];
		assert_eq!(pick_version(&versions, "Sodium-Fabric-0.5.8+mc1.20.4.jar", "x").unwrap().id, "b");
		assert_eq!(pick_version(&versions, "renamed.jar", "abc").unwrap().id, "b");
		assert_eq!(pick_version(&versions, "sodium-0.5.7.jar", "x").unwrap().id, "a");
		assert!(pick_version(&versions, "sodium-0.5.9.jar", "x").is_none());
	}

	#[test]
	fn version_tokens_need_boundaries() {
		assert!(contains_token("sodium-0.5.8+mc1.20", "0.5.8"));
		assert!(!contains_token("sodium-0.5.80", "0.5.8"));
		assert!(!contains_token("sodium-0.5.8.1", "0.5.8"));
		assert!(!contains_token("sodium-10.5.8", "0.5.8"));
		assert!(!contains_token("sodium-beta", "beta"));
	}
}
