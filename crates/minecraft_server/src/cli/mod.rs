mod commands;
mod create;
mod run;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mcserver", about = "Minecraft server installer, manager, and runner")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Minecraft server interactively
    Create {
        /// Directory to create the server in (defaults to current directory)
        #[arg(short, long)]
        dir: Option<String>,
    },
    /// Start a Minecraft server
    Run {
        /// Server directory (defaults to current directory)
        #[arg(short, long)]
        dir: Option<String>,
    },
    /// Send a command to a running server
    Send {
        /// The command to send
        command: String,
        /// Server directory (defaults to current directory)
        #[arg(short, long)]
        dir: Option<String>,
    },
    /// Stop a running server gracefully
    Stop {
        /// Server directory (defaults to current directory)
        #[arg(short, long)]
        dir: Option<String>,
    },
    /// Force kill a running server
    Kill {
        /// Server directory (defaults to current directory)
        #[arg(short, long)]
        dir: Option<String>,
    },
    /// List available Minecraft versions
    Versions {
        /// Show only release versions (no snapshots)
        #[arg(short, long)]
        releases_only: bool,
    },
    /// Show server status
    Status {
        /// Server directory (defaults to current directory)
        #[arg(short, long)]
        dir: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Create { dir } => {
            create::create_server(dir).await?;
        }
        Commands::Run { dir } => {
            run::run_server(dir).await?;
        }
        Commands::Send { command, dir } => {
            commands::send_command(dir, &command).await?;
        }
        Commands::Stop { dir } => {
            commands::stop_server(dir).await?;
        }
        Commands::Kill { dir } => {
            commands::kill_server(dir).await?;
        }
        Commands::Versions { releases_only } => {
            commands::list_versions(releases_only).await?;
        }
        Commands::Status { dir } => {
            commands::show_status(dir)?;
        }
    }

    Ok(())
}
