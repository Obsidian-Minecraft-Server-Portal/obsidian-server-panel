use crate::app_db;
use crate::server::server_data::ServerData;
use anyhow::Result;
use chrono::FixedOffset;
use cron_tab::AsyncCron;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct BackupScheduler {
    cron: AsyncCron<FixedOffset>,
    scheduled_servers: Arc<RwLock<HashMap<u64, usize>>>, // server_id -> job_id mapping
}

impl BackupScheduler {
    pub fn new() -> Self {
        let utc_tz = FixedOffset::east_opt(0).unwrap(); // UTC timezone
        let cron = AsyncCron::new(utc_tz);

        Self { cron, scheduled_servers: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn start(&mut self) {
        log::info!("Starting backup scheduler");
        self.cron.start().await;

        // Schedule backups for all servers with backup enabled
        if let Err(e) = self.schedule_all_servers().await {
            log::error!("Failed to schedule backups for all servers: {}", e);
        }
    }

    pub async fn stop(&mut self) {
        log::info!("Stopping backup scheduler");
        self.cron.stop().await;

        // Clear scheduled servers
        let mut scheduled = self.scheduled_servers.write().await;
        scheduled.clear();
    }

    pub async fn schedule_server_backup(&mut self, server: &ServerData) -> Result<()> {
        if !server.backup_enabled {
            log::debug!("Backup not enabled for server: {}", server.name);
            return Ok(());
        }

        // Remove existing schedule if any
        self.unschedule_server_backup(server.id).await?;

        let server_id = server.id;
        let user_id = server.owner_id;
        let server_name = server.name.clone();
        let cron_expression = server.backup_cron.clone();

        log::info!("Scheduling backup for server '{}' with cron: '{}'", server_name, cron_expression);

        // Create the backup job
        let job_result = self
            .cron
            .add_fn(&cron_expression, move || {
                let server_id = server_id;
                let server_name = server_name.clone();

                async move {
                    log::info!("Executing scheduled backup for server: {}", server_name);

                    match Self::execute_backup(server_id, user_id).await {
                        Ok(()) => {
                            log::info!("Scheduled backup completed successfully for server: {}", server_name);
                        }
                        Err(e) => {
                            log::error!("Scheduled backup failed for server '{}': {}", server_name, e);
                        }
                    }
                }
            })
            .await;

        match job_result {
            Ok(job_id) => {
                // Store the job ID for this server
                let mut scheduled = self.scheduled_servers.write().await;
                scheduled.insert(server.id, job_id);
                log::info!("Successfully scheduled backup for server: {}", server.name);
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to schedule backup for server '{}': {}", server.name, e);
                Err(anyhow::anyhow!("Failed to schedule backup: {}", e))
            }
        }
    }

    pub async fn unschedule_server_backup(&mut self, server_id: u64) -> Result<()> {
        let mut scheduled = self.scheduled_servers.write().await;

        if let Some(job_id) = scheduled.remove(&server_id) {
            self.cron.remove(job_id).await;
            log::info!("Unscheduled backup for server ID: {}", server_id);
        }

        Ok(())
    }

    pub async fn reschedule_server_backup(&mut self, server: &ServerData) -> Result<()> {
        log::info!("Rescheduling backup for server: {}", server.name);
        self.unschedule_server_backup(server.id).await?;
        self.schedule_server_backup(server).await
    }

    async fn schedule_all_servers(&mut self) -> Result<()> {
        let pool = app_db::open_pool().await?;
        let servers = ServerData::list_all_with_pool(&pool).await?;
        pool.close().await;

        for server in servers {
            if server.backup_enabled {
                if let Err(e) = self.schedule_server_backup(&server).await {
                    log::error!("Failed to schedule backup for server '{}': {}", server.name, e);
                }
            }
        }

        Ok(())
    }

    async fn execute_backup(server_id: u64, user_id: u64) -> Result<(), String> {
        let server = match ServerData::get(server_id, user_id).await {
            Ok(Some(server)) => server,
            Ok(None) => {
                return Err("Server not found".to_string());
            }
            Err(e) => {
                return Err(format!("Failed to get server: {}", e));
            }
        };

        // Execute the backup
        server.backup().await
    }

    pub async fn get_scheduled_servers(&self) -> Vec<u64> {
        let scheduled = self.scheduled_servers.read().await;
        scheduled.keys().cloned().collect()
    }

    pub async fn is_server_scheduled(&self, server_id: u64) -> bool {
        let scheduled = self.scheduled_servers.read().await;
        scheduled.contains_key(&server_id)
    }
}

use tokio::sync::Mutex as AsyncMutex;

// Global backup scheduler instance
static BACKUP_SCHEDULER: std::sync::OnceLock<Arc<AsyncMutex<BackupScheduler>>> = std::sync::OnceLock::new();

pub fn get_backup_scheduler() -> Arc<AsyncMutex<BackupScheduler>> {
    BACKUP_SCHEDULER.get_or_init(|| Arc::new(AsyncMutex::new(BackupScheduler::new()))).clone()
}

pub async fn initialize_backup_scheduler() -> Result<()> {
    let scheduler = get_backup_scheduler();
    let mut scheduler_guard = scheduler.lock().await;
    scheduler_guard.start().await;
    Ok(())
}

pub async fn shutdown_backup_scheduler() -> Result<()> {
    let scheduler = get_backup_scheduler();
    let mut scheduler_guard = scheduler.lock().await;
    scheduler_guard.stop().await;
    Ok(())
}

pub async fn schedule_server_backup(server: &ServerData) -> Result<()> {
    let scheduler = get_backup_scheduler();
    let mut scheduler_guard = scheduler.lock().await;
    scheduler_guard.schedule_server_backup(server).await
}

pub async fn unschedule_server_backup(server_id: u64) -> Result<()> {
    let scheduler = get_backup_scheduler();
    let mut scheduler_guard = scheduler.lock().await;
    scheduler_guard.unschedule_server_backup(server_id).await
}

pub async fn reschedule_server_backup(server: &ServerData) -> Result<()> {
    let scheduler = get_backup_scheduler();
    let mut scheduler_guard = scheduler.lock().await;
    scheduler_guard.reschedule_server_backup(server).await
}

pub async fn is_server_scheduled(server_id: u64) -> bool {
    let scheduler = get_backup_scheduler();
    let scheduler_guard = scheduler.lock().await;
    scheduler_guard.is_server_scheduled(server_id).await
}
