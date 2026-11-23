use std::future::Future;
use std::hash::Hash;
use std::time::Duration;

use moka::future::Cache;
use obsidian_common::Result;

use crate::model::{Facets, Project, ProjectDetail, ProjectVersion, SearchPage, SearchQuery, VersionFilter};
use crate::platform::{Kind, Platform};

const SEARCH_TTL: Duration = Duration::from_secs(10 * 60);
const FACETS_TTL: Duration = Duration::from_secs(6 * 60 * 60);
const PROJECT_TTL: Duration = Duration::from_secs(30 * 60);
const VERSIONS_TTL: Duration = Duration::from_secs(10 * 60);
const CHANGELOG_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const DETAIL_TTL: Duration = Duration::from_secs(30 * 60);
const ENTRY_CAPACITY: u64 = 512;

#[derive(Clone)]
pub(crate) struct ContentCache {
	pub(crate) search: Cache<(Platform, SearchQuery), SearchPage>,
	pub(crate) facets: Cache<(Platform, Kind), Facets>,
	pub(crate) projects: Cache<(Platform, String), Project>,
	pub(crate) versions: Cache<(Platform, String, VersionFilter), Vec<ProjectVersion>>,
	pub(crate) changelogs: Cache<(Platform, String, String), Option<String>>,
	pub(crate) details: Cache<(Platform, String), ProjectDetail>,
}

impl ContentCache {
	pub(crate) fn new() -> Self {
		Self {
			search: build(SEARCH_TTL),
			facets: build(FACETS_TTL),
			projects: build(PROJECT_TTL),
			versions: build(VERSIONS_TTL),
			changelogs: build(CHANGELOG_TTL),
			details: build(DETAIL_TTL),
		}
	}
}

fn build<K, V>(ttl: Duration) -> Cache<K, V>
where
	K: Hash + Eq + Send + Sync + 'static,
	V: Clone + Send + Sync + 'static,
{
	Cache::builder().time_to_live(ttl).max_capacity(ENTRY_CAPACITY).build()
}

pub(crate) async fn get_or_fetch<K, V, F>(cache: &Cache<K, V>, key: K, fetch: F) -> Result<V>
where
	K: Hash + Eq + Clone + Send + Sync + 'static,
	V: Clone + Send + Sync + 'static,
	F: Future<Output = Result<V>>,
{
	if let Some(value) = cache.get(&key).await {
		return Ok(value);
	}
	let value = fetch.await?;
	cache.insert(key, value.clone()).await;
	Ok(value)
}

#[cfg(test)]
mod tests {
	use std::sync::atomic::{AtomicUsize, Ordering};

	use super::*;

	#[test]
	fn repeated_keys_reach_the_upstream_once() {
		let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
		runtime.block_on(async {
			let cache: Cache<&'static str, u32> = build(Duration::from_secs(60));
			let calls = AtomicUsize::new(0);
			for _ in 0..3 {
				let value = get_or_fetch(&cache, "key", async {
					calls.fetch_add(1, Ordering::SeqCst);
					Ok(7)
				})
				.await
				.unwrap();
				assert_eq!(value, 7);
			}
			assert_eq!(calls.load(Ordering::SeqCst), 1);
		});
	}
}
