use crate::server::server_data::ServerData;
use crate::server::server_status::ServerStatus;
use anyhow::Result;
use log::{debug, error};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_interactive::AsynchronousInteractiveProcess;

static ACTIVE_SERVERS: OnceLock<Arc<Mutex<HashMap<u64, u32>>>> = OnceLock::new();

impl ServerData {
    pub async fn start_server(&mut self) -> Result<()> {
        self.status = ServerStatus::Starting;
        self.save().await?;
        let arguments = format!("{} {} {}", &self.java_args, &self.server_jar, &self.minecraft_args);

        let self_clone = self.clone();
        let pid = AsynchronousInteractiveProcess::new(&self.java_executable)
            .with_argument(arguments)
            .with_working_directory(self.get_directory_path())
            .process_exit_callback(move |exit_code| {
                let mut self_clone = self_clone.clone();
                tokio::spawn(async move {
                    debug!("Server exited with code {}", exit_code);
                    if exit_code != 0 {
                        if let Err(e) = self_clone.remove_server_crashed().await{
                            error!("Failed to remove server from list of running servers, you may need to restart the web panel in order to prevent against memory leaks: {}", e);
                        }
                        return;
                    }
                    if let Err(e) = self_clone.remove_server().await{
                        error!("Failed to remove server from list of running servers, you may need to restart the web panel in order to prevent against memory leaks: {}", e);
                    }
                });
            })
            .start()
            .await?;
        let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        servers.lock().await.insert(self.id, pid);
        debug!("Server started with pid {}", pid);
        self.last_started = Some(chrono::Utc::now().timestamp() as u64);
        self.save().await?;

        let process = AsynchronousInteractiveProcess::get_process_by_pid(pid).await.expect("Server not running");

        let hang_duration = Duration::from_secs(120); // 2 minutes

        let id = self.id;
        let owner_id = self.owner_id;
        tokio::spawn(async move {
            tokio::time::sleep(hang_duration).await;
            let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
            let servers = servers.lock().await;
            let pid = *servers.get(&id).expect("Server not running");
            let process = AsynchronousInteractiveProcess::get_process_by_pid(pid).await.expect("Server not running");
            if !process.is_process_running().await {
                return;
            }
            let server = ServerData::get(id, owner_id).await.expect("Server not found").expect("Server not found");
            if server.status == ServerStatus::Starting {}
        });

        loop {
            let line = process.receive_output().await?;
            if let Some(line) = line {
                if line.contains("Done (") && line.contains(r#")! For help, type "help""#) {
                    self.status = ServerStatus::Running;
                    self.save().await?;
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn stop_server(&mut self) -> Result<()> {
        self.send_command("stop").await?;

        let mut iterations = 0;
        loop {
            let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
            let servers = servers.lock().await;
            if servers.get(&self.id).is_none() {
                break;
            }
            iterations += 1;
            tokio::time::sleep(Duration::from_secs(1)).await;
            if iterations >= 30 {
                // 30 seconds
                return self.kill_server().await;
            }
        }

        self.remove_server().await?;

        Ok(())
    }

    pub async fn kill_server(&mut self) -> Result<()> {
        let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let servers = servers.lock().await;
        let pid = *servers.get(&self.id).expect("Server not running");
        let process = AsynchronousInteractiveProcess::get_process_by_pid(pid).await.expect("Server not running");
        process.shutdown(Duration::from_secs(10)).await?; // tries to gracefully shutdown, but after 10 seconds it will kill the process.
        self.remove_server().await?;

        Ok(())
    }

    pub async fn restart_server(&mut self) -> Result<()> {
        self.stop_server().await?;
        tokio::time::sleep(Duration::from_secs(3)).await;
        self.start_server().await?;
        Ok(())
    }

    async fn remove_server(&mut self) -> Result<()> {
        let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let mut servers = servers.lock().await;
        servers.remove(&self.id);
        self.status = ServerStatus::Stopped;
        self.save().await?;

        Ok(())
    }

    async fn remove_server_crashed(&mut self) -> Result<()> {
        let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let mut servers = servers.lock().await;
        servers.remove(&self.id);
        self.status = ServerStatus::Crashed;
        self.save().await?;

        Ok(())
    }

    pub async fn send_command(&self, command: impl Into<String>) -> Result<()> {
        let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let servers = servers.lock().await;
        let pid = *servers.get(&self.id).expect("Server not running");
        let process = AsynchronousInteractiveProcess::get_process_by_pid(pid).await.expect("Server not running");
        process.send_input(command).await?;

        Ok(())
    }

    pub async fn attach_to_stdout(&self, sender: tokio::sync::mpsc::Sender<actix_web_lab::sse::Event>) -> Result<()> {
        let servers = ACTIVE_SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
        let servers = servers.lock().await;
        let pid = *servers.get(&self.id).expect("Server not running");
        let process = AsynchronousInteractiveProcess::get_process_by_pid(pid).await.expect("Server not running");
        loop {
            let line = process.receive_output().await?;
            if let Some(line) = line {
                let message = actix_web_lab::sse::Data::new(line).event(format!("server-{}-console", self.id));
                if sender.send(message.into()).await.is_err() {
                    break;
                }
            }
        }

        Ok(())
    }
}
