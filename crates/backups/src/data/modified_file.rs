#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModifiedFile {
	/// The path of the modified file.
	pub path: String,
	/// The content of the file before modification (if available).
	pub content_before: Option<Vec<u8>>,
	/// The content of the file after modification (if available).
	pub content_after: Option<Vec<u8>>,
}