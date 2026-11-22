use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};

use obsidian_common::Result;
use serde::de::DeserializeOwned;
use tokio::io::AsyncWriteExt;

use crate::progress::Stage;
use crate::upstream;

const USER_AGENT: &str = "Obsidian-Panel";
const REPORT_STEP: f32 = 0.01;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
	reqwest::Client::builder().user_agent(USER_AGENT).build().expect("static http client configuration is valid")
});
static PARTIAL_ID: AtomicU64 = AtomicU64::new(0);

pub async fn get_json<T: DeserializeOwned>(url: &str) -> Result<T> {
	get(url).await?.json().await.map_err(upstream)
}

pub async fn get_text(url: &str) -> Result<String> {
	get(url).await?.text().await.map_err(upstream)
}

async fn get(url: &str) -> Result<reqwest::Response> {
	CLIENT.get(url).send().await.and_then(reqwest::Response::error_for_status).map_err(upstream)
}

pub async fn download(url: &str, destination: &Path, stage: Stage<'_>, label: &str) -> Result<()> {
	if let Some(parent) = destination.parent() {
		tokio::fs::create_dir_all(parent).await?;
	}
	let partial = partial_path(destination);
	let result = fetch_into(url, &partial, stage, label).await;
	match result {
		Ok(()) => Ok(tokio::fs::rename(&partial, destination).await?),
		Err(error) => {
			let _ = tokio::fs::remove_file(&partial).await;
			Err(error)
		}
	}
}

fn partial_path(destination: &Path) -> PathBuf {
	let id = PARTIAL_ID.fetch_add(1, Ordering::Relaxed);
	let mut name = destination.file_name().unwrap_or_default().to_os_string();
	name.push(format!(".{}-{id}.part", std::process::id()));
	destination.with_file_name(name)
}

async fn fetch_into(url: &str, partial: &Path, stage: Stage<'_>, label: &str) -> Result<()> {
	let mut response = get(url).await?;
	let total = response.content_length().filter(|total| *total > 0);
	let mut file = tokio::fs::File::create(partial).await?;
	let mut received = 0u64;
	let mut reported = 0.0f32;
	while let Some(chunk) = response.chunk().await.map_err(upstream)? {
		file.write_all(&chunk).await?;
		received += chunk.len() as u64;
		if let Some(total) = total {
			let fraction = received as f32 / total as f32;
			if fraction - reported >= REPORT_STEP {
				reported = fraction;
				stage.report(fraction, label);
			}
		}
	}
	file.flush().await?;
	Ok(())
}
