use crate::actions::actions_data::{ActionData, ActionStatus, ActionType, TrackedAction};
use anyhow::Result;

impl ActionData {
    pub async fn create(
        user_id: i64,
        tracker_id: String,
        action_type: ActionType,
        details: Option<String>,
    ) -> Result<Self> {
        let tracked_action = TrackedAction::create(user_id, tracker_id, action_type, None, details)
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(tracked_action.into())
    }

    pub async fn get_by_tracker_id(tracker_id: &str) -> Result<Option<Self>> {
        match TrackedAction::get_by_tracker_id(tracker_id) {
            Ok(Some(tracked_action)) => Ok(Some(tracked_action.into())),
            Ok(None) => Ok(None),
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    }

    pub async fn get_by_user_id(user_id: i64) -> Result<Vec<Self>> {
        let tracked_actions = TrackedAction::get_by_user_id(user_id)
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(tracked_actions.into_iter().map(|a| a.into()).collect())
    }

    pub async fn get_active_by_user_id(user_id: i64) -> Result<Vec<Self>> {
        let tracked_actions = TrackedAction::get_active_by_user_id(user_id)
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(tracked_actions.into_iter().map(|a| a.into()).collect())
    }

    pub async fn update_progress(&self, progress: i64) -> Result<()> {
        if let Ok(Some(mut tracked_action)) = TrackedAction::get_by_tracker_id(&self.tracker_id) {
            tracked_action.update_progress(progress).map_err(|e| anyhow::anyhow!(e))?;
        }
        Ok(())
    }

    pub async fn update_status(
        &self,
        status: ActionStatus,
        details: Option<String>,
    ) -> Result<()> {
        if let Ok(Some(mut tracked_action)) = TrackedAction::get_by_tracker_id(&self.tracker_id) {
            tracked_action.update_status(status, details).map_err(|e| anyhow::anyhow!(e))?;
        }
        Ok(())
    }

    pub async fn delete_completed_by_user_id(user_id: i64) -> Result<()> {
        TrackedAction::delete_completed_by_user_id(user_id)
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn delete_by_tracker_id(tracker_id: &str) -> Result<()> {
        TrackedAction::delete_by_tracker_id(tracker_id)
            .map_err(|e| anyhow::anyhow!(e))
    }
}