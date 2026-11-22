use std::process::Stdio;

use obsidian_common::{InstanceId, Paths, Result};
use process_wrap::std::{ChildWrapper, CommandWrap};

pub const DATA_DIR_ENV: &str = "OBSIDIAN_DATA_DIR";

#[cfg(windows)]
const DETACHED_PROCESS: u32 = 0x0000_0008;
#[cfg(windows)]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

pub fn spawn_detached(paths: &Paths, id: InstanceId) -> Result<Box<dyn ChildWrapper>> {
	let exe = std::env::current_exe()?;
	let mut command = CommandWrap::with_new(exe, |command| {
		command
			.args(["runner", "--instance"])
			.arg(id.to_string())
			.env(DATA_DIR_ENV, &paths.data_dir)
			.stdin(Stdio::null())
			.stdout(Stdio::null())
			.stderr(Stdio::null());
		#[cfg(windows)]
		std::os::windows::process::CommandExt::creation_flags(command, DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
	});
	#[cfg(unix)]
	command.wrap(process_wrap::std::ProcessSession);
	Ok(command.spawn()?)
}
