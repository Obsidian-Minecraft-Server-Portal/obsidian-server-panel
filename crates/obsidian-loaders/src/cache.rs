use std::future::Future;
use std::sync::LazyLock;
use std::time::Duration;

use moka::future::Cache;
use obsidian_common::Result;

use crate::unshare_error;

const TTL: Duration = Duration::from_secs(15 * 60);
const CAPACITY: u64 = 256;

static VERSIONS: LazyLock<Cache<String, Vec<String>>> =
	LazyLock::new(|| Cache::builder().time_to_live(TTL).max_capacity(CAPACITY).build());

pub async fn versions(key: String, fetch: impl Future<Output = Result<Vec<String>>>) -> Result<Vec<String>> {
	VERSIONS.try_get_with(key, fetch).await.map_err(unshare_error)
}
