use clap::Parser;

#[derive(Parser)]
#[command(version, author, about, long_about = None)]
pub struct CommandLineArgs {
	#[arg(long, short, default_value = "80")]
	pub port: u16,

	#[arg(long, default_value = "false")]
	pub forward_webpanel: bool,

	/// MySQL connection string (e.g., mysql://user:password@host:port/database)
	/// Can also be set via DATABASE_URL environment variable
	#[arg(long, short = 'd', env = "DATABASE_URL")]
	pub database_url: String,
}
