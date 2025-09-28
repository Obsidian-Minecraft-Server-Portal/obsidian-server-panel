use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock, atomic::{AtomicU64, Ordering}};
use chrono::{DateTime, Utc};

static ACTION_STORE: OnceLock<Arc<RwLock<HashMap<i64, Vec<TrackedAction>>>>> = OnceLock::new();
static ID_COUNTER: AtomicU64 = AtomicU64::new(1);

pub fn get_action_store() -> &'static Arc<RwLock<HashMap<i64, Vec<TrackedAction>>>> {
    ACTION_STORE.get_or_init(|| Arc::new(RwLock::new(HashMap::new())))
}

fn generate_id() -> String {
    ID_COUNTER.fetch_add(1, Ordering::SeqCst).to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedAction {
    pub id: String,
    pub user_id: i64,
    pub tracker_id: String,
    pub action_type: ActionType,
    pub status: ActionStatus,
    pub progress: i64,
    pub description: Option<String>,
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

// Legacy struct for API compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionData {
    pub id: i64,
    pub user_id: i64,
    pub tracker_id: String,
    pub action_type: String,
    pub status: String,
    pub progress: i64,
    pub details: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    Archive,
    Extract,
    Move,
    Copy,
    Upload,
    BackupCreate,
    ModDownload,
}

impl ActionType {
    pub fn as_str(&self) -> &str {
        match self {
            ActionType::Archive => "archive",
            ActionType::Extract => "extract",
            ActionType::Move => "move",
            ActionType::Copy => "copy",
            ActionType::Upload => "upload",
            ActionType::BackupCreate => "backup_create",
            ActionType::ModDownload => "mod_download",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "archive" => Some(ActionType::Archive),
            "extract" => Some(ActionType::Extract),
            "move" => Some(ActionType::Move),
            "copy" => Some(ActionType::Copy),
            "upload" => Some(ActionType::Upload),
            "backup_create" => Some(ActionType::BackupCreate),
            "mod_download" => Some(ActionType::ModDownload),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionStatus {
    InProgress,
    Completed,
    Failed,
}

impl ActionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ActionStatus::InProgress => "in_progress",
            ActionStatus::Completed => "completed",
            ActionStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "in_progress" => Some(ActionStatus::InProgress),
            "completed" => Some(ActionStatus::Completed),
            "failed" => Some(ActionStatus::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateActionRequest {
    pub tracker_id: String,
    pub action_type: ActionType,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateActionRequest {
    pub status: Option<ActionStatus>,
    pub progress: Option<i64>,
    pub details: Option<serde_json::Value>,
}

impl TrackedAction {
    pub fn new(
        user_id: i64,
        tracker_id: String,
        action_type: ActionType,
        description: Option<String>,
        details: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: generate_id(),
            user_id,
            tracker_id,
            action_type,
            status: ActionStatus::InProgress,
            progress: 0,
            description,
            details,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    pub fn create(
        user_id: i64,
        tracker_id: String,
        action_type: ActionType,
        description: Option<String>,
        details: Option<String>,
    ) -> Result<Self, String> {
        let action = Self::new(user_id, tracker_id, action_type, description, details);
        let store = get_action_store();
        
        match store.write() {
            Ok(mut store_guard) => {
                let user_actions = store_guard.entry(user_id).or_insert_with(Vec::new);
                user_actions.push(action.clone());
                Ok(action)
            }
            Err(_) => Err("Failed to acquire write lock".to_string()),
        }
    }

    pub fn get_by_tracker_id(tracker_id: &str) -> Result<Option<Self>, String> {
        let store = get_action_store();
        
        match store.read() {
            Ok(store_guard) => {
                for user_actions in store_guard.values() {
                    if let Some(action) = user_actions.iter().find(|a| a.tracker_id == tracker_id) {
                        return Ok(Some(action.clone()));
                    }
                }
                Ok(None)
            }
            Err(_) => Err("Failed to acquire read lock".to_string()),
        }
    }

    pub fn get_by_user_id(user_id: i64) -> Result<Vec<Self>, String> {
        let store = get_action_store();
        
        match store.read() {
            Ok(store_guard) => {
                Ok(store_guard.get(&user_id).cloned().unwrap_or_default())
            }
            Err(_) => Err("Failed to acquire read lock".to_string()),
        }
    }

    pub fn get_active_by_user_id(user_id: i64) -> Result<Vec<Self>, String> {
        let store = get_action_store();
        
        match store.read() {
            Ok(store_guard) => {
                let actions = store_guard.get(&user_id).cloned().unwrap_or_default();
                Ok(actions.into_iter().filter(|a| a.status == ActionStatus::InProgress).collect())
            }
            Err(_) => Err("Failed to acquire read lock".to_string()),
        }
    }

    pub fn update_progress(&mut self, progress: i64) -> Result<(), String> {
        self.progress = progress;
        self.updated_at = Utc::now();
        self.update_in_store()
    }

    pub fn update_status(&mut self, status: ActionStatus, details: Option<String>) -> Result<(), String> {
        self.status = status.clone();
        if let Some(details) = details {
            self.details = Some(details);
        }
        self.updated_at = Utc::now();
        
        if status == ActionStatus::Completed || status == ActionStatus::Failed {
            self.completed_at = Some(Utc::now());
        }
        
        self.update_in_store()
    }

    fn update_in_store(&self) -> Result<(), String> {
        let store = get_action_store();
        
        match store.write() {
            Ok(mut store_guard) => {
                if let Some(user_actions) = store_guard.get_mut(&self.user_id) {
                    if let Some(existing_action) = user_actions.iter_mut().find(|a| a.tracker_id == self.tracker_id) {
                        *existing_action = self.clone();
                    }
                }
                Ok(())
            }
            Err(_) => Err("Failed to acquire write lock".to_string()),
        }
    }

    pub fn delete_completed_by_user_id(user_id: i64) -> Result<(), String> {
        let store = get_action_store();
        
        match store.write() {
            Ok(mut store_guard) => {
                if let Some(user_actions) = store_guard.get_mut(&user_id) {
                    user_actions.retain(|a| a.status == ActionStatus::InProgress);
                }
                Ok(())
            }
            Err(_) => Err("Failed to acquire write lock".to_string()),
        }
    }

    pub fn delete_by_tracker_id(tracker_id: &str) -> Result<(), String> {
        let store = get_action_store();
        
        match store.write() {
            Ok(mut store_guard) => {
                for user_actions in store_guard.values_mut() {
                    user_actions.retain(|a| a.tracker_id != tracker_id);
                }
                Ok(())
            }
            Err(_) => Err("Failed to acquire write lock".to_string()),
        }
    }
}

impl From<TrackedAction> for ActionData {
    fn from(tracked: TrackedAction) -> Self {
        Self {
            id: tracked.id.parse().unwrap_or(0),
            user_id: tracked.user_id,
            tracker_id: tracked.tracker_id,
            action_type: tracked.action_type.as_str().to_string(),
            status: tracked.status.as_str().to_string(),
            progress: tracked.progress,
            details: tracked.details,
            created_at: tracked.created_at.to_rfc3339(),
            updated_at: tracked.updated_at.to_rfc3339(),
            completed_at: tracked.completed_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_type_as_str() {
        assert_eq!(ActionType::Archive.as_str(), "archive");
        assert_eq!(ActionType::Extract.as_str(), "extract");
        assert_eq!(ActionType::Move.as_str(), "move");
        assert_eq!(ActionType::Copy.as_str(), "copy");
        assert_eq!(ActionType::Upload.as_str(), "upload");
        assert_eq!(ActionType::BackupCreate.as_str(), "backup_create");
        assert_eq!(ActionType::ModDownload.as_str(), "mod_download");
    }

    #[test]
    fn test_action_type_from_str() {
        assert_eq!(ActionType::from_str("archive"), Some(ActionType::Archive));
        assert_eq!(ActionType::from_str("extract"), Some(ActionType::Extract));
        assert_eq!(ActionType::from_str("move"), Some(ActionType::Move));
        assert_eq!(ActionType::from_str("copy"), Some(ActionType::Copy));
        assert_eq!(ActionType::from_str("upload"), Some(ActionType::Upload));
        assert_eq!(ActionType::from_str("backup_create"), Some(ActionType::BackupCreate));
        assert_eq!(ActionType::from_str("mod_download"), Some(ActionType::ModDownload));
        assert_eq!(ActionType::from_str("invalid"), None);
    }

    #[test]
    fn test_action_status_as_str() {
        assert_eq!(ActionStatus::InProgress.as_str(), "in_progress");
        assert_eq!(ActionStatus::Completed.as_str(), "completed");
        assert_eq!(ActionStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn test_action_status_from_str() {
        assert_eq!(ActionStatus::from_str("in_progress"), Some(ActionStatus::InProgress));
        assert_eq!(ActionStatus::from_str("completed"), Some(ActionStatus::Completed));
        assert_eq!(ActionStatus::from_str("failed"), Some(ActionStatus::Failed));
        assert_eq!(ActionStatus::from_str("invalid"), None);
    }

    #[test]
    fn test_action_type_round_trip() {
        let types = vec![
            ActionType::Archive,
            ActionType::Extract,
            ActionType::Move,
            ActionType::Copy,
            ActionType::Upload,
            ActionType::BackupCreate,
            ActionType::ModDownload,
        ];

        for action_type in types {
            let str_repr = action_type.as_str();
            let parsed = ActionType::from_str(str_repr).unwrap();
            assert_eq!(action_type.as_str(), parsed.as_str());
        }
    }

    #[test]
    fn test_action_status_round_trip() {
        let statuses = vec![
            ActionStatus::InProgress,
            ActionStatus::Completed,
            ActionStatus::Failed,
        ];

        for status in statuses {
            let str_repr = status.as_str();
            let parsed = ActionStatus::from_str(str_repr).unwrap();
            assert_eq!(status.as_str(), parsed.as_str());
        }
    }
}