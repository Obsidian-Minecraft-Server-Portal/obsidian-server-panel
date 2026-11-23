use std::collections::HashMap;

use obsidian_common::{CommonError, Result};
use obsidian_content::{Compatibility, CompatibilityEntry, InstalledContent, PlannedDependency, ProjectVersion, Upgrade};

use crate::model::{Action, StartRequest};

pub(crate) struct Plan {
	pub upgrades: Vec<PlannedUpgrade>,
	pub disable: Vec<InstalledContent>,
}

pub(crate) struct PlannedUpgrade {
	pub item: InstalledContent,
	pub version: ProjectVersion,
	pub dependencies: Vec<PlannedDependency>,
}

impl Plan {
	pub(crate) fn new(entries: Vec<CompatibilityEntry>, request: &StartRequest) -> Result<Self> {
		let decisions: HashMap<i64, Action> = request.decisions.iter().map(|decision| (decision.mod_id, decision.action)).collect();
		let mut plan = Self {
			upgrades: Vec::new(),
			disable: Vec::new(),
		};
		for entry in entries {
			match entry.compatibility {
				Compatibility::Upgradable(upgrade) => {
					let Upgrade { version, dependencies, .. } = *upgrade;
					plan.upgrades.push(PlannedUpgrade {
						item: entry.item,
						version,
						dependencies,
					});
				}
				Compatibility::Unavailable { .. } => {
					if request.disable_unavailable {
						plan.disable.push(entry.item);
					}
				}
				Compatibility::DeclaredCompatible { .. } => {}
				Compatibility::NeedsDecision { .. } => match decisions.get(&entry.item.id) {
					Some(Action::Disable) => plan.disable.push(entry.item),
					Some(Action::Keep) => {}
					None => {
						return Err(CommonError::Invalid(format!(
							"choose whether to keep or disable {}",
							entry.item.filename
						)));
					}
				},
			}
		}
		Ok(plan)
	}
}

#[cfg(test)]
mod tests {
	use obsidian_content::{Compatibility, CompatibilityEntry, InstalledContent, Kind};

	use super::Plan;
	use crate::model::{Action, Decision, StartRequest};

	fn entry(id: i64, compatibility: Compatibility) -> CompatibilityEntry {
		CompatibilityEntry {
			item: InstalledContent {
				id,
				filename: format!("mod-{id}.jar"),
				name: None,
				project_id: None,
				version_id: None,
				platform: None,
				kind: Kind::Mod,
				enabled: true,
				instance_id: 1,
			},
			compatibility,
		}
	}

	fn request(disable_unavailable: bool, decisions: Vec<Decision>) -> StartRequest {
		StartRequest {
			mc_version: "1.21.4".into(),
			loader_version: None,
			backup: true,
			disable_unavailable,
			decisions,
		}
	}

	fn undecided(range: Option<&str>) -> Compatibility {
		Compatibility::NeedsDecision {
			reason: "unknown".into(),
			minecraft_range: range.map(str::to_owned),
		}
	}

	#[test]
	fn undecided_mods_block_the_plan() {
		let entries = vec![entry(1, undecided(None))];
		assert!(Plan::new(entries, &request(true, Vec::new())).is_err());
	}

	#[test]
	fn decisions_route_each_flagged_mod() {
		let entries = vec![entry(1, undecided(None)), entry(2, undecided(Some("[1.19,1.20)"))), entry(3, undecided(None))];
		let decisions = vec![
			Decision {
				mod_id: 1,
				action: Action::Disable,
			},
			Decision {
				mod_id: 2,
				action: Action::Keep,
			},
			Decision {
				mod_id: 3,
				action: Action::Disable,
			},
		];
		let plan = Plan::new(entries, &request(false, decisions)).unwrap();
		assert_eq!(plan.disable.iter().map(|item| item.id).collect::<Vec<_>>(), vec![1, 3]);
		assert!(plan.upgrades.is_empty());
	}

	#[test]
	fn unavailable_mods_follow_the_bulk_choice() {
		let entries = || vec![entry(1, Compatibility::Unavailable { reason: "gone".into() })];
		assert!(Plan::new(entries(), &request(false, Vec::new())).unwrap().disable.is_empty());
		assert_eq!(Plan::new(entries(), &request(true, Vec::new())).unwrap().disable.len(), 1);
	}

	#[test]
	fn declared_compatible_mods_stay_enabled() {
		let entries = vec![entry(
			1,
			Compatibility::DeclaredCompatible {
				reason: "declared".into(),
				minecraft_range: ">=1.21".into(),
			},
		)];
		let plan = Plan::new(entries, &request(true, Vec::new())).unwrap();
		assert!(plan.disable.is_empty() && plan.upgrades.is_empty());
	}
}
