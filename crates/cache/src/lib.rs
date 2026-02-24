//! A generic, thread-safe, in-memory TTL cache for async Rust.
//!
//! Provides `TtlCache<K, V>` -- a hash map where each entry has a
//! configurable time-to-live. Expired entries are lazily evicted on
//! access. The cache is backed by `tokio::sync::RwLock` for safe
//! concurrent use from async tasks.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// A single cached entry with a TTL.
struct CacheEntry<V> {
    value: V,
    inserted_at: Instant,
    ttl: Duration,
}

impl<V> CacheEntry<V> {
    fn is_expired(&self) -> bool {
        self.inserted_at.elapsed() > self.ttl
    }
}

/// A generic, thread-safe, in-memory cache with per-entry TTL expiration.
///
/// Entries are lazily evicted on access. The cache is backed by a
/// `tokio::sync::RwLock<HashMap>` for safe concurrent access from async tasks.
///
/// # Example
///
/// ```
/// use cache::TtlCache;
/// use std::time::Duration;
///
/// # async fn example() {
/// let cache: TtlCache<String, String> = TtlCache::new(Duration::from_secs(300));
/// cache.insert("key".into(), "value".into()).await;
///
/// assert_eq!(cache.get(&"key".into()).await, Some("value".into()));
/// # }
/// ```
pub struct TtlCache<K, V> {
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    default_ttl: Duration,
}

impl<K, V> TtlCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Creates a new cache with the given default TTL for entries.
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    /// Retrieves a cached value by key. Returns `None` if the key is missing
    /// or the entry has expired. Expired entries are removed on access.
    pub async fn get(&self, key: &K) -> Option<V> {
        // Fast path: read lock
        {
            let entries = self.entries.read().await;
            if let Some(entry) = entries.get(key) {
                if !entry.is_expired() {
                    return Some(entry.value.clone());
                }
            } else {
                return None;
            }
        }
        // Entry exists but is expired -- remove it
        {
            let mut entries = self.entries.write().await;
            entries.remove(key);
        }
        None
    }

    /// Inserts a value with the cache's default TTL.
    pub async fn insert(&self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl).await;
    }

    /// Inserts a value with a custom TTL.
    pub async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let mut entries = self.entries.write().await;
        entries.insert(
            key,
            CacheEntry {
                value,
                inserted_at: Instant::now(),
                ttl,
            },
        );
    }

    /// Removes a specific entry from the cache.
    pub async fn invalidate(&self, key: &K) {
        let mut entries = self.entries.write().await;
        entries.remove(key);
    }

    /// Removes all entries from the cache.
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }

    /// Removes all expired entries from the cache.
    pub async fn cleanup(&self) {
        let mut entries = self.entries.write().await;
        entries.retain(|_, entry| !entry.is_expired());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_insert_and_get() {
        let cache: TtlCache<String, String> = TtlCache::new(Duration::from_secs(60));
        cache.insert("key".to_string(), "value".to_string()).await;
        let result = cache.get(&"key".to_string()).await;
        assert_eq!(result, Some("value".to_string()));
    }

    #[tokio::test]
    async fn test_get_missing_key() {
        let cache: TtlCache<String, String> = TtlCache::new(Duration::from_secs(60));
        let result = cache.get(&"missing".to_string()).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache: TtlCache<String, String> = TtlCache::new(Duration::from_millis(50));
        cache.insert("key".to_string(), "value".to_string()).await;
        assert!(cache.get(&"key".to_string()).await.is_some());
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(cache.get(&"key".to_string()).await.is_none());
    }

    #[tokio::test]
    async fn test_custom_ttl() {
        let cache: TtlCache<String, String> = TtlCache::new(Duration::from_secs(60));
        cache
            .insert_with_ttl(
                "short".to_string(),
                "value".to_string(),
                Duration::from_millis(50),
            )
            .await;
        assert!(cache.get(&"short".to_string()).await.is_some());
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(cache.get(&"short".to_string()).await.is_none());
    }

    #[tokio::test]
    async fn test_invalidate() {
        let cache: TtlCache<String, String> = TtlCache::new(Duration::from_secs(60));
        cache.insert("key".to_string(), "value".to_string()).await;
        cache.invalidate(&"key".to_string()).await;
        assert!(cache.get(&"key".to_string()).await.is_none());
    }

    #[tokio::test]
    async fn test_clear() {
        let cache: TtlCache<String, String> = TtlCache::new(Duration::from_secs(60));
        cache.insert("a".to_string(), "1".to_string()).await;
        cache.insert("b".to_string(), "2".to_string()).await;
        cache.clear().await;
        assert!(cache.get(&"a".to_string()).await.is_none());
        assert!(cache.get(&"b".to_string()).await.is_none());
    }

    #[tokio::test]
    async fn test_cleanup_removes_expired() {
        let cache: TtlCache<String, String> = TtlCache::new(Duration::from_millis(50));
        cache.insert("expired".to_string(), "old".to_string()).await;
        tokio::time::sleep(Duration::from_millis(100)).await;
        cache
            .insert_with_ttl(
                "fresh".to_string(),
                "new".to_string(),
                Duration::from_secs(60),
            )
            .await;
        cache.cleanup().await;
        assert!(cache.get(&"expired".to_string()).await.is_none());
        assert!(cache.get(&"fresh".to_string()).await.is_some());
    }

    #[tokio::test]
    async fn test_overwrite_existing_key() {
        let cache: TtlCache<String, String> = TtlCache::new(Duration::from_secs(60));
        cache
            .insert("key".to_string(), "original".to_string())
            .await;
        cache
            .insert("key".to_string(), "updated".to_string())
            .await;
        assert_eq!(
            cache.get(&"key".to_string()).await,
            Some("updated".to_string())
        );
    }
}
