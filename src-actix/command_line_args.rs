use clap::Parser;

#[derive(Parser)]
#[command(version, author, about, long_about = None)]
pub struct CommandLineArgs{
	#[arg(long, short, default_value = "80")]
	pub port: u16,
	#[arg(long, default_value = "false")]
	pub forward_webpanel: bool,
}