//! Keeps each instance's pack loaded between renders and detail draws, so its jars are read once per pack key.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use obsidian_common::{InstanceId, Result};
use obsidian_worldmap::colour::SurfaceCache;
use obsidian_worldmap::render::{Pack, PackSources};

use crate::job::{Sources, other};

/// A pack that has not been used for this long is dropped, so its assets leave memory with the viewers.
const IDLE_LIMIT: Duration = Duration::from_secs(10 * 60);

/// A loaded pack and the surface textures decoded for it, shared by renders and detail draws.
#[derive(Clone)]
pub struct PackHandle {
    pub pack: Arc<Pack>,
    pub surfaces: Arc<Mutex<SurfaceCache>>,
}

struct Entry {
    key: String,
    handle: PackHandle,
    used: Instant,
}

#[derive(Default)]
pub struct PackCache {
    entries: Mutex<HashMap<InstanceId, Entry>>,
}

impl PackCache {
    /// The pack for `id` at `key`. Reads the jars when it is not held yet, so call it from a blocking thread.
    pub fn get(&self, id: InstanceId, key: &str, sources: &Sources) -> Result<PackHandle> {
        if let Some(handle) = self.lookup(id, key) {
            return Ok(handle);
        }
        let pack = Pack::load(&PackSources {
            client_jar: &sources.client_jar,
            server_jar: &sources.server_jar,
            mod_jars: &sources.mod_jars,
            blocks_report: &sources.blocks_report,
        })
        .map_err(other)?;
        let handle = PackHandle {
            pack: Arc::new(pack),
            surfaces: Arc::new(Mutex::new(SurfaceCache::default())),
        };
        self.entries.lock().unwrap_or_else(PoisonError::into_inner).insert(
            id,
            Entry {
                key: key.to_owned(),
                handle: handle.clone(),
                used: Instant::now(),
            },
        );
        Ok(handle)
    }

    pub fn release_idle(&self) {
        self.entries
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .retain(|_, entry| entry.used.elapsed() < IDLE_LIMIT);
    }

    fn lookup(&self, id: InstanceId, key: &str) -> Option<PackHandle> {
        let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        entries.retain(|_, entry| entry.used.elapsed() < IDLE_LIMIT);
        let entry = entries.get_mut(&id).filter(|entry| entry.key == key)?;
        entry.used = Instant::now();
        Some(entry.handle.clone())
    }
}
