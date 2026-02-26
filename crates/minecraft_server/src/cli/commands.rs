use minecraft_server::ServerConfig;
use std::path::PathBuf;

pub async fn send_command(dir: Option<String>, command: &str) -> anyhow::Result<()> {
    let directory = PathBuf::from(dir.unwrap_or_else(|| ".".to_string()));
    let config_path = directory.join("server_config.json");

    if !config_path.exists() {
        anyhow::bail!(
            "No server_config.json found in {}",
            directory.display()
        );
    }

    let config = ServerConfig::load(&config_path)?;
    let manager = minecraft_server::ServerManager::new(config, minecraft_server::NoOpHandler);
    manager.send_command(command).await?;
    println!("Command sent: {}", command);
    Ok(())
}

pub async fn stop_server(dir: Option<String>) -> anyhow::Result<()> {
    send_command(dir, "stop").await
}

pub async fn kill_server(dir: Option<String>) -> anyhow::Result<()> {
    let _ = dir;
    // Force kill requires PID tracking which is process-local.
    // For now, just send "stop" and let the user Ctrl+C if needed.
    println!("Force kill is only available when the server was started from this process.");
    println!("Use 'mcserver stop' for graceful shutdown.");
    Ok(())
}

pub async fn list_versions(releases_only: bool) -> anyhow::Result<()> {
    println!("Fetching Minecraft versions...\n");

    let versions = if releases_only {
        minecraft_server::versions::list_release_versions().await?
    } else {
        minecraft_server::versions::list_minecraft_versions().await?
    };

    for version in &versions {
        println!("{:<15} {}", version.id, version.release_type);
    }

    println!("\nTotal: {} versions", versions.len());
    Ok(())
}

pub fn show_status(dir: Option<String>) -> anyhow::Result<()> {
    let directory = PathBuf::from(dir.unwrap_or_else(|| ".".to_string()));
    let config_path = directory.join("server_config.json");

    if !config_path.exists() {
        anyhow::bail!(
            "No server_config.json found in {}",
            directory.display()
        );
    }

    let config = ServerConfig::load(&config_path)?;
    println!("Server: {}", config.name);
    println!("Directory: {}", config.directory.display());
    println!("Minecraft: {}", config.minecraft_version);
    println!("Type: {}", config.server_type);
    println!("JAR: {}", config.server_jar);
    println!("Memory: {}-{}GB", config.min_memory_gb, config.max_memory_gb);
    println!("Java: {}", config.java_executable);

    if let Some(loader) = &config.loader_version {
        println!("Loader: {}", loader);
    }

    // Check if EULA is accepted
    if minecraft_server::eula::is_eula_accepted(&config.directory) {
        println!("EULA: Accepted");
    } else {
        println!("EULA: Not accepted");
    }

    Ok(())
}
