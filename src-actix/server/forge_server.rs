use crate::server::server_data::ServerData;
use crate::server::server_status::ServerStatus;
use anyhow::Result;
use log::{debug, error};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_interactive::AsynchronousInteractiveProcess;

impl ServerData {
    pub(crate) fn is_forge_installer(&self) -> bool {
        if let Some(server_type) = &self.server_type
            && *server_type == crate::server::server_type::ServerType::Forge {
                return self.server_jar.contains("installer") || self.server_jar.contains("forge") && !self.server_jar.contains("server");
            }
        false
    }
    pub(crate) async fn install_forge_server(&mut self) -> Result<()> {
        self.status = ServerStatus::Starting;
        self.save().await?;
        debug!("Installing forge server {}", self.id);

        let directory_path = self.get_directory_path().canonicalize()?;
        let self_clone = self.clone();

        // Create the process builder
        let process_builder = AsynchronousInteractiveProcess::new(&self.java_executable)
            .with_argument("-jar")
            .with_argument(&self.server_jar)
            .with_argument("-installServer");

        let pid = process_builder
			.with_working_directory(&directory_path)
			.process_exit_callback(move |exit_code| {
				let mut self_clone = self_clone.clone();
				tokio::spawn(async move {
					debug!("Server exited with code {}", exit_code);
					if let Err(e) = self_clone.parse_start_script().await{
						error!("Failed to parse start script: {}", e);
					}
					if exit_code != 0 {
						if let Err(e) = self_clone.remove_server_crashed().await {
							error!("Failed to remove server from list of running servers, you may need to restart the web panel in order to prevent against memory leaks: {}", e);
						}

						return;
					}
					if let Err(e) = self_clone.remove_server().await {
						error!("Failed to remove server from list of running servers, you may need to restart the web panel in order to prevent against memory leaks: {}", e);
					}
				});
			})
			.start()
			.await?;

        let servers = crate::server::server_actions::ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        servers.lock().await.insert(self.id, pid);

        Ok(())
    }

    async fn parse_start_script(&mut self) -> Result<()> {
        let directory_path = self.get_directory_path().canonicalize()?;
        let start_script = std::fs::read_to_string(directory_path.join(format!("run.{}", if cfg!(windows) { "bat" } else { "sh" })))?;
        let lines = start_script.lines();
        for line in lines {
            if !line.starts_with("java") {
                continue;
            }
            if let Some(index) = line.find("@libraries") {
                self.java_args = line[index..].split_whitespace().next().unwrap_or(line[index..].trim()).to_string();
                self.server_jar = String::new();
                self.save().await?;
                break;
            }
        }

        Ok(())
    }
}
