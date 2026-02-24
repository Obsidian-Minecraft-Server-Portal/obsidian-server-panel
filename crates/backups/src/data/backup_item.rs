#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BackupItem {
	/// The unique identifier for the backup item.
	pub id: String,
	/// The timestamp when the backup was created.
	pub timestamp: chrono::DateTime<chrono::Utc>,
	/// A description or notes about the backup item.
	pub description: String,
}