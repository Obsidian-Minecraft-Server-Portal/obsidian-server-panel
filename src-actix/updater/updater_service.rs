// update_service.rs
use crate::updater::updater_data::{UpdateStatus, Updater};
use actix_web::{HttpResponse, Result as ActixResult};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct UpdateService {
    pub(crate) updater: Arc<Updater>,
    status: Arc<RwLock<UpdateStatus>>,
}

impl UpdateService {
    pub fn new() -> Self {
        Self { updater: Arc::new(Updater::new()), status: Arc::new(RwLock::new(UpdateStatus::NoUpdateAvailable)) }
    }

    /// Check for updates endpoint
    pub async fn check_updates(&self) -> ActixResult<HttpResponse> {
        match self.updater.check_for_updates().await {
            Ok(status) => {
                *self.status.write().await = status.clone();

                let response = match status {
                    UpdateStatus::UpdateAvailable { version, .. } => json!({
                        "update_available": true,
                        "current_version": self.updater.current_version(),
                        "latest_version": version,
                        "status": "available"
                    }),
                    UpdateStatus::NoUpdateAvailable => json!({
                        "update_available": false,
                        "current_version": self.updater.current_version(),
                        "status": "up_to_date"
                    }),
                    _ => json!({
                        "update_available": false,
                        "current_version": self.updater.current_version(),
                        "status": "checking"
                    }),
                };

                Ok(HttpResponse::Ok().json(response))
            }
            Err(e) => {
                log::error!("Update check failed: {}", e);
                Ok(HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to check for updates",
                    "message": e.to_string()
                })))
            }
        }
    }

    /// Perform update endpoint
    pub async fn perform_update(&self) -> ActixResult<HttpResponse> {
        let current_status = self.status.read().await.clone();

        match current_status {
            UpdateStatus::UpdateAvailable { download_url, .. } => {
                *self.status.write().await = UpdateStatus::UpdateInProgress;

                // Perform update in background
                let updater = Arc::clone(&self.updater);
                let status = Arc::clone(&self.status);
                let download_url = download_url.to_owned();

                tokio::spawn(async move {
                    log::info!("Starting update");
                    match updater.perform_update(&download_url).await {
                        Ok(UpdateStatus::UpdateCompleted) => {
                            *status.write().await = UpdateStatus::UpdateCompleted;

                            // Wait a moment then restart
                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                            if let Err(e) = updater.restart_application().await {
                                log::error!("Failed to restart application: {}", e);
                                *status.write().await = UpdateStatus::UpdateFailed(format!("Restart failed: {}", e));
                            }
                        }
                        Ok(status_result) => {
                            *status.write().await = status_result; // Use the cloned status, not self_clone.status
                        }
                        Err(e) => {
                            log::error!("Update failed: {}", e);
                            *status.write().await = UpdateStatus::UpdateFailed(e.to_string());
                        }
                    }
                });

                Ok(HttpResponse::Ok().json(json!({
                    "status": "update_started",
                    "message": "Update is in progress. The application will restart automatically."
                })))
            }
            _ => Ok(HttpResponse::BadRequest().json(json!({
                "error": "No update available",
                "status": "no_update"
            }))),
        }
    }

    /// Get current update status
    pub async fn get_status(&self) -> ActixResult<HttpResponse> {
        let status = self.status.read().await.clone();

        let response = match status {
            UpdateStatus::NoUpdateAvailable => json!({
                "status": "up_to_date",
                "current_version": self.updater.current_version()
            }),
            UpdateStatus::UpdateAvailable { version, .. } => json!({
                "status": "available",
                "current_version": self.updater.current_version(),
                "latest_version": version
            }),
            UpdateStatus::UpdateInProgress => json!({
                "status": "updating",
                "current_version": self.updater.current_version()
            }),
            UpdateStatus::UpdateCompleted => json!({
                "status": "completed",
                "current_version": self.updater.current_version()
            }),
            UpdateStatus::UpdateFailed(error) => json!({
                "status": "failed",
                "error": error,
                "current_version": self.updater.current_version()
            }),
        };

        Ok(HttpResponse::Ok().json(response))
    }
}
