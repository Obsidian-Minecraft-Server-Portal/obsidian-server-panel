pub mod reset_password;

use clap::{Parser, Subcommand};
use obsidian_common::InstanceId;

use crate::service::ServiceAction;

#[derive(Debug, Parser)]
#[command(name = "obsidian", version, about = "Obsidian Minecraft server panel")]
pub struct Cli {
	#[command(subcommand)]
	pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
	/// Run the web panel (default)
	Serve {
		/// Run without the system tray icon
		#[arg(long)]
		headless: bool,
	},
	/// Run the supervisor process for one instance
	Runner {
		#[arg(long)]
		instance: InstanceId,
	},
	/// Install, remove, start or stop the OS service
	Service {
		#[command(subcommand)]
		action: ServiceAction,
	},
	/// Replace a user's password with a generated temporary one
	ResetPassword { username: String },
}
