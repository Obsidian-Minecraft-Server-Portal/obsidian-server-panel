use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, WorldmapError>;

#[derive(Debug, thiserror::Error)]
pub enum WorldmapError {
	#[error("{}: {reason}", .path.display())]
	Region { path: PathBuf, reason: String },
	#[error("{0}")]
	Chunk(String),
	#[error("unknown chunk compression id {0}")]
	Compression(u8),
	#[error(transparent)]
	Io(#[from] std::io::Error),
	#[error(transparent)]
	Nbt(#[from] fastnbt::error::Error),
	#[error(transparent)]
	Image(#[from] image::ImageError),
}
