use dialoguer::{Input, Select};
use minecraft_server::{ServerConfig, ServerManager, ServerType};
use std::path::PathBuf;

/// A CLI event handler that prints status and output to the terminal.
pub(crate) struct CliEventHandler;

impl minecraft_server::ServerEventHandler for CliEventHandler {
    async fn on_event(&self, event: minecraft_server::ServerEvent) {
        match event {
            minecraft_server::ServerEvent::StatusChanged { ref status } => {
                println!("[Status] {}", status);
            }
            minecraft_server::ServerEvent::ConsoleOutput { ref line } => {
                println!("{}", line);
            }
            minecraft_server::ServerEvent::InstallProgress {
                ref file,
                completed,
                total,
                current,
            } => {
                if completed {
                    println!("[Install] {}/{} - {} (done)", current, total, file);
                } else {
                    println!("[Install] {}/{} - {}...", current, total, file);
                }
            }
            minecraft_server::ServerEvent::Started => {
                println!("[Server] Started successfully!");
            }
            minecraft_server::ServerEvent::Stopped => {
                println!("[Server] Stopped.");
            }
            minecraft_server::ServerEvent::Crashed { exit_code } => {
                eprintln!("[Server] Crashed with exit code {}", exit_code);
            }
            minecraft_server::ServerEvent::JavaVersionError => {
                eprintln!("[Server] Java version mismatch! Please update your Java installation.");
            }
        }
    }
}

pub async fn create_server(dir: Option<String>) -> anyhow::Result<()> {
    println!("=== Minecraft Server Creator ===\n");

    // Server name
    let name: String = Input::new()
        .with_prompt("Server name")
        .default("Minecraft Server".to_string())
        .interact_text()?;

    // Directory
    let directory = match dir {
        Some(d) => PathBuf::from(d),
        None => {
            let dir_str: String = Input::new()
                .with_prompt("Server directory")
                .default(".".to_string())
                .interact_text()?;
            PathBuf::from(dir_str)
        }
    };

    // Minecraft version
    println!("\nFetching available Minecraft versions...");
    let versions = minecraft_server::versions::list_release_versions().await?;
    let version_names: Vec<&str> = versions.iter().take(20).map(|v| v.id.as_str()).collect();

    let version_idx = Select::new()
        .with_prompt("Minecraft version")
        .items(&version_names)
        .default(0)
        .interact()?;
    let minecraft_version = version_names[version_idx].to_string();

    // Server type
    let type_options = ["Vanilla", "Fabric", "Forge", "NeoForge", "Custom"];
    let type_idx = Select::new()
        .with_prompt("Server type")
        .items(&type_options)
        .default(0)
        .interact()?;

    let server_type = match type_idx {
        0 => ServerType::Vanilla,
        1 => ServerType::Fabric,
        2 => ServerType::Forge,
        3 => ServerType::NeoForge,
        _ => ServerType::Custom,
    };

    // Loader version (if modded)
    let loader_version = match server_type {
        ServerType::Fabric | ServerType::Forge | ServerType::NeoForge => {
            let version: String = Input::new()
                .with_prompt("Loader version (leave empty for latest)")
                .allow_empty(true)
                .interact_text()?;
            if version.is_empty() {
                None
            } else {
                Some(version)
            }
        }
        _ => None,
    };

    // Memory settings
    let max_memory: u8 = Input::new()
        .with_prompt("Maximum memory (GB)")
        .default(2)
        .interact_text()?;

    let min_memory: u8 = Input::new()
        .with_prompt("Minimum memory (GB)")
        .default(1)
        .interact_text()?;

    // Java executable
    let java_executable: String = Input::new()
        .with_prompt("Java executable")
        .default("java".to_string())
        .interact_text()?;

    // Build config
    let config = ServerConfig {
        name: name.clone(),
        directory: directory.clone(),
        java_executable,
        java_args: String::new(),
        max_memory_gb: max_memory,
        min_memory_gb: min_memory,
        minecraft_args: String::new(),
        server_jar: "server.jar".to_string(),
        minecraft_version,
        server_type,
        loader_version,
    };

    // Install
    println!("\nInstalling server...");
    let mut manager = ServerManager::new(config, CliEventHandler);
    manager.install().await?;

    // Save config
    let config_path = directory.join("server_config.json");
    manager.config().save(&config_path)?;

    println!("\nServer '{}' created successfully!", name);
    println!("Config saved to: {}", config_path.display());
    println!("Run with: mcserver run --dir {}", directory.display());

    Ok(())
}
