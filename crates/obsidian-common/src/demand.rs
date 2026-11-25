//! Which topics connected clients are subscribed to, so that producers can skip work nobody is watching.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

#[derive(Debug, Clone, Default)]
pub struct TopicDemand {
	sessions: Arc<Mutex<HashMap<u64, HashSet<String>>>>,
	next: Arc<AtomicU64>,
}

impl TopicDemand {
	/// One lease per connection. Its topics count as watched until the lease is dropped.
	pub fn lease(&self) -> DemandLease {
		DemandLease {
			id: self.next.fetch_add(1, Ordering::Relaxed),
			demand: self.clone(),
		}
	}

	pub fn is_watched(&self, topic: &str) -> bool {
		self.sessions().values().any(|topics| topics.contains(topic))
	}

	pub fn watched_topics(&self) -> HashSet<String> {
		self.sessions().values().flatten().cloned().collect()
	}

	fn sessions(&self) -> std::sync::MutexGuard<'_, HashMap<u64, HashSet<String>>> {
		self.sessions.lock().unwrap_or_else(PoisonError::into_inner)
	}
}

#[derive(Debug)]
pub struct DemandLease {
	id: u64,
	demand: TopicDemand,
}

impl DemandLease {
	pub fn set(&self, topics: &HashSet<String>) {
		let mut sessions = self.demand.sessions();
		if topics.is_empty() {
			sessions.remove(&self.id);
		} else {
			sessions.insert(self.id, topics.clone());
		}
	}
}

impl Drop for DemandLease {
	fn drop(&mut self) {
		self.demand.sessions().remove(&self.id);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_topic_is_watched_while_a_lease_holds_it() {
		let demand = TopicDemand::default();
		let lease = demand.lease();
		assert!(!demand.is_watched("instance.1.players"));
		lease.set(&HashSet::from(["instance.1.players".to_owned()]));
		assert!(demand.is_watched("instance.1.players"));
		assert_eq!(demand.watched_topics(), HashSet::from(["instance.1.players".to_owned()]));
		lease.set(&HashSet::new());
		assert!(!demand.is_watched("instance.1.players"));
	}

	#[test]
	fn dropping_a_lease_releases_its_topics() {
		let demand = TopicDemand::default();
		{
			let lease = demand.lease();
			lease.set(&HashSet::from(["instance.2.players".to_owned()]));
		}
		assert!(!demand.is_watched("instance.2.players"));
	}
}
