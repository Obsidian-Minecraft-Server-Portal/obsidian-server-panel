use serde::{Deserialize, Serialize};

use crate::Permission;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
	Viewer,
	Operator,
	Moderator,
	Admin,
	CoOwner,
}

impl Role {
	pub const ALL: [Self; 5] = [
		Self::Viewer,
		Self::Operator,
		Self::Moderator,
		Self::Admin,
		Self::CoOwner,
	];

	pub fn as_str(self) -> &'static str {
		match self {
			Self::Viewer => "viewer",
			Self::Operator => "operator",
			Self::Moderator => "moderator",
			Self::Admin => "admin",
			Self::CoOwner => "co_owner",
		}
	}

	pub fn parse(name: &str) -> Option<Self> {
		Self::ALL.into_iter().find(|role| role.as_str() == name)
	}

	pub fn permissions(self) -> Permission {
		let viewer = Permission::CONSOLE_READ | Permission::FILES_READ;
		let operator = viewer
			| Permission::CONSOLE_SEND
			| Permission::POWER_START
			| Permission::POWER_STOP
			| Permission::POWER_RESTART;
		let moderator = operator
			| Permission::POWER_KILL
			| Permission::FILES_WRITE
			| Permission::FILES_DELETE
			| Permission::CONTENT_INSTALL
			| Permission::CONTENT_REMOVE
			| Permission::PLAYERS_MANAGE
			| Permission::BACKUPS_CREATE
			| Permission::BACKUPS_DOWNLOAD;
		let admin = moderator
			| Permission::FILES_SFTP
			| Permission::BACKUPS_RESTORE
			| Permission::BACKUPS_DELETE
			| Permission::SCHEDULES_MANAGE
			| Permission::SETTINGS_EDIT;
		match self {
			Self::Viewer => viewer,
			Self::Operator => operator,
			Self::Moderator => moderator,
			Self::Admin => admin,
			Self::CoOwner => Permission::all(),
		}
	}
}
