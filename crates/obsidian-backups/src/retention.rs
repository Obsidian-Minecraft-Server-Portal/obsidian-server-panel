use std::collections::HashSet;
use std::hash::Hash;

use chrono::{DateTime, Datelike, Timelike, Utc};

use crate::model::{RetentionPolicy, SnapshotInfo};

pub(crate) fn expired(snapshots: &[SnapshotInfo], keep: &RetentionPolicy) -> Vec<String> {
	let mut newest_first: Vec<&SnapshotInfo> = snapshots.iter().collect();
	newest_first.sort_unstable_by(|a, b| b.created_at.cmp(&a.created_at).then_with(|| b.id.cmp(&a.id)));
	let mut kept: HashSet<&str> = HashSet::new();
	if newest_first.is_empty() || keep.is_disabled() {
		return Vec::new();
	}
	kept.insert(newest_first[0].id.as_str());
	keep_per_bucket(&newest_first, keep.keep_hourly, |t| (t.year(), t.ordinal(), t.hour()), &mut kept);
	keep_per_bucket(&newest_first, keep.keep_daily, |t| (t.year(), t.ordinal()), &mut kept);
	keep_per_bucket(
		&newest_first,
		keep.keep_weekly,
		|t| {
			let week = t.iso_week();
			(week.year(), week.week())
		},
		&mut kept,
	);
	keep_per_bucket(&newest_first, keep.keep_monthly, |t| (t.year(), t.month()), &mut kept);
	newest_first
		.into_iter()
		.filter(|snapshot| !kept.contains(snapshot.id.as_str()))
		.map(|snapshot| snapshot.id.clone())
		.collect()
}

fn keep_per_bucket<'a, K: Eq + Hash>(
	newest_first: &[&'a SnapshotInfo],
	limit: u32,
	bucket: impl Fn(&DateTime<Utc>) -> K,
	kept: &mut HashSet<&'a str>,
) {
	let mut seen = HashSet::new();
	for snapshot in newest_first {
		if seen.len() == limit as usize {
			break;
		}
		if seen.insert(bucket(&snapshot.created_at)) {
			kept.insert(snapshot.id.as_str());
		}
	}
}
