use super::{backup_db, backup_service};
use crate::server::server_data::ServerData;
use anyhow::Result;
use log::{debug, error, info, warn};
use obsidian_scheduler::callback::CallbackTimer;
use obsidian_scheduler::timer_trait::Timer;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Backup scheduler that runs scheduled backups
pub struct BackupScheduler {
    pool: SqlitePool,
    timer: Option<Arc<CallbackTimer>>,
    running: Arc<RwLock<bool>>,
}

impl BackupScheduler {
    /// Create a new backup scheduler
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            timer: None,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the backup scheduler
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting backup scheduler");

        let pool = self.pool.clone();
        let running = self.running.clone();

        // Create a timer that runs every minute to check for scheduled backups
        let timer = CallbackTimer::new(
            move |_handle| {
                let pool = pool.clone();
                let running = running.clone();
                async move {
                    // Check if already running
                    let is_running = *running.read().await;
                    if is_running {
                        debug!("Backup scheduler already processing, skipping this cycle");
                        return Ok(());
                    }

                    // Set running flag
                    {
                        let mut running_write = running.write().await;
                        *running_write = true;
                    }

                    // Process scheduled backups
                    if let Err(e) = Self::process_scheduled_backups(&pool).await {
                        error!("Error processing scheduled backups: {}", e);
                    }

                    // Clear running flag
                    {
                        let mut running_write = running.write().await;
                        *running_write = false;
                    }

                    Ok(())
                }
            },
            Duration::from_secs(60), // Check every minute
        );

        timer.start().await?;
        self.timer = Some(timer);

        info!("Backup scheduler started successfully");
        Ok(())
    }

    /// Stop the backup scheduler
    pub async fn stop(&mut self) {
        info!("Stopping backup scheduler");
        if let Some(timer) = &self.timer {
            let _ = timer.stop().await;
        }
        self.timer = None;
    }

    /// Check if the scheduler is running
    pub async fn is_running(&self) -> bool {
        if let Some(timer) = &self.timer {
            timer.is_running().await
        } else {
            false
        }
    }

    /// Process all scheduled backups that are due
    async fn process_scheduled_backups(pool: &SqlitePool) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        // Get all enabled schedules
        let schedules = backup_db::list_all_enabled_schedules(pool).await?;

        debug!("Found {} enabled backup schedules", schedules.len());

        for schedule in schedules {
            // Check if this schedule is due
            let should_run = if let Some(next_run) = schedule.next_run {
                next_run <= now
            } else {
                // First run - calculate next run based on interval
                true
            };

            if should_run {
                info!(
                    "Running scheduled backup for server {} (schedule ID: {})",
                    schedule.server_id, schedule.id
                );

                // Get the server
                let server = match ServerData::get_with_pool(schedule.server_id as u64, pool).await {
                    Ok(Some(server)) => server,
                    Ok(None) => {
                        warn!(
                            "Server {} not found for backup schedule {}",
                            schedule.server_id, schedule.id
                        );
                        continue;
                    }
                    Err(e) => {
                        error!(
                            "Failed to get server {} for backup schedule {}: {}",
                            schedule.server_id, schedule.id, e
                        );
                        continue;
                    }
                };

                // Perform the backup
                let description = Some(format!(
                    "Scheduled {} at {}",
                    match schedule.backup_type {
                        super::backup_data::BackupType::Incremental => "incremental backup",
                        super::backup_data::BackupType::WorldOnly => "world backup",
                    },
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
                ));

                match backup_service::perform_backup(
                    &server,
                    schedule.backup_type,
                    description,
                )
                .await
                {
                    Ok(commit_id) => {
                        info!(
                            "Scheduled backup completed successfully: commit ID {}",
                            commit_id
                        );

                        // Note: Retention policy management would require implementing
                        // backup cleanup logic using git operations, which is not currently
                        // supported by obsidian-backups. Consider implementing this in the future.
                        if schedule.retention_days.is_some() {
                            debug!(
                                "Retention policy is set to {} days but automatic cleanup is not yet implemented",
                                schedule.retention_days.unwrap()
                            );
                        }
                    }
                    Err(e) => {
                        error!("Failed to create scheduled backup: {}", e);
                    }
                }

                // Calculate next run time based on interval
                let next_run = Self::calculate_next_run(
                    schedule.interval_amount,
                    &schedule.interval_unit,
                    now,
                )?;

                // Update schedule with last_run and next_run
                if let Err(e) =
                    backup_db::update_schedule_run_times(schedule.id, now, next_run, pool).await
                {
                    error!("Failed to update schedule run times: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Calculate the next run time based on interval
    fn calculate_next_run(
        interval_amount: i64,
        interval_unit: &str,
        from_time: i64,
    ) -> Result<i64> {
        let seconds = match interval_unit {
            "hours" => interval_amount * 3600,
            "days" => interval_amount * 86400,
            "weeks" => interval_amount * 604800,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid interval unit: {}. Must be 'hours', 'days', or 'weeks'",
                    interval_unit
                ));
            }
        };

        Ok(from_time + seconds)
    }
}
