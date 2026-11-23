use std::collections::HashSet;
use std::sync::{Arc, Mutex, PoisonError};

use obsidian_common::{CommonError, InstanceId, Result};

pub struct MigrationLock {
	id: InstanceId,
	migrating: Arc<Mutex<HashSet<InstanceId>>>,
}

impl MigrationLock {
	pub(crate) fn acquire(migrating: &Arc<Mutex<HashSet<InstanceId>>>, id: InstanceId) -> Result<Self> {
		let inserted = migrating.lock().unwrap_or_else(PoisonError::into_inner).insert(id);
		if !inserted {
			return Err(CommonError::Conflict("a migration is already running for this server".into()));
		}
		Ok(Self {
			id,
			migrating: Arc::clone(migrating),
		})
	}

	pub fn instance_id(&self) -> InstanceId {
		self.id
	}
}

impl Drop for MigrationLock {
	fn drop(&mut self) {
		self.migrating.lock().unwrap_or_else(PoisonError::into_inner).remove(&self.id);
	}
}
