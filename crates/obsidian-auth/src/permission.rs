use bitflags::bitflags;

bitflags! {
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
	pub struct Permission: u32 {
		const CONSOLE_READ = 1 << 0;
		const CONSOLE_SEND = 1 << 1;
		const POWER_START = 1 << 2;
		const POWER_STOP = 1 << 3;
		const POWER_RESTART = 1 << 4;
		const POWER_KILL = 1 << 5;
		const FILES_READ = 1 << 6;
		const FILES_WRITE = 1 << 7;
		const FILES_DELETE = 1 << 8;
		const FILES_SFTP = 1 << 9;
		const CONTENT_INSTALL = 1 << 10;
		const CONTENT_REMOVE = 1 << 11;
		const BACKUPS_CREATE = 1 << 12;
		const BACKUPS_RESTORE = 1 << 13;
		const BACKUPS_DOWNLOAD = 1 << 14;
		const BACKUPS_DELETE = 1 << 15;
		const SCHEDULES_MANAGE = 1 << 16;
		const PLAYERS_MANAGE = 1 << 17;
		const SETTINGS_EDIT = 1 << 18;
		const MEMBERS_MANAGE = 1 << 19;
		const MAP_VIEW = 1 << 20;
		const MAP_PLAYERS = 1 << 21;
	}
}

pub const PERMISSION_NAMES: [(&str, Permission); 22] = [
	("console.read", Permission::CONSOLE_READ),
	("console.send", Permission::CONSOLE_SEND),
	("power.start", Permission::POWER_START),
	("power.stop", Permission::POWER_STOP),
	("power.restart", Permission::POWER_RESTART),
	("power.kill", Permission::POWER_KILL),
	("files.read", Permission::FILES_READ),
	("files.write", Permission::FILES_WRITE),
	("files.delete", Permission::FILES_DELETE),
	("files.sftp", Permission::FILES_SFTP),
	("content.install", Permission::CONTENT_INSTALL),
	("content.remove", Permission::CONTENT_REMOVE),
	("backups.create", Permission::BACKUPS_CREATE),
	("backups.restore", Permission::BACKUPS_RESTORE),
	("backups.download", Permission::BACKUPS_DOWNLOAD),
	("backups.delete", Permission::BACKUPS_DELETE),
	("schedules.manage", Permission::SCHEDULES_MANAGE),
	("players.manage", Permission::PLAYERS_MANAGE),
	("settings.edit", Permission::SETTINGS_EDIT),
	("members.manage", Permission::MEMBERS_MANAGE),
	("map.view", Permission::MAP_VIEW),
	("map.players", Permission::MAP_PLAYERS),
];

impl Permission {
	pub fn by_name(name: &str) -> Option<Self> {
		PERMISSION_NAMES
			.iter()
			.find(|(known, _)| *known == name)
			.map(|(_, flag)| *flag)
	}

	pub fn names(self) -> Vec<&'static str> {
		PERMISSION_NAMES
			.iter()
			.filter(|(_, flag)| self.contains(*flag))
			.map(|(name, _)| *name)
			.collect()
	}
}
