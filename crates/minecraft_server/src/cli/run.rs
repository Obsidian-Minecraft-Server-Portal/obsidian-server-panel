use super::create::CliEventHandler;
use minecraft_server::{ServerConfig, ServerManager};
use std::path::PathBuf;

pub async fn run_server(dir: Option<String>) -> anyhow::Result<()> {
    let directory = PathBuf::from(dir.unwrap_or_else(|| ".".to_string()));
    let config_path = directory.join("server_config.json");

    if !config_path.exists() {
        anyhow::bail!(
            "No server_config.json found in {}. Run 'mcserver create' first.",
            directory.display()
        );
    }

    let config = ServerConfig::load(&config_path)?;
    println!("Starting server '{}'...", config.name);

    let mut manager = ServerManager::new(config, CliEventHandler);
    manager.start().await?;

    // Subscribe to output and print to terminal
    let mut rx = manager.subscribe_output().await?;
    println!("Server is running. Press Ctrl+C to stop.\n");

    // Handle Ctrl+C for graceful shutdown
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        let _ = shutdown_tx.send(());
    });

    loop {
        tokio::select! {
            line = rx.recv() => {
                match line {
                    Some(line) => println!("{}", line),
                    None => {
                        println!("\nServer process ended.");
                        break;
                    }
                }
            }
            _ = &mut shutdown_rx => {
                println!("\nStopping server...");
                manager.stop().await?;
                // Wait a moment for the server to stop
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                break;
            }
        }
    }

    Ok(())
}
