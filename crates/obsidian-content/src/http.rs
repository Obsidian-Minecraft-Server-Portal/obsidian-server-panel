use std::fmt::Display;
use std::time::Duration;

use obsidian_common::{CommonError, Result};
use reqwest::{RequestBuilder, StatusCode, Url};
use serde::de::DeserializeOwned;

const USER_AGENT: &str = concat!(
	"Obsidian/",
	env!("CARGO_PKG_VERSION"),
	" (+https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel)"
);

pub fn client() -> reqwest::Client {
	reqwest::Client::builder()
		.user_agent(USER_AGENT)
		.connect_timeout(Duration::from_secs(15))
		.timeout(Duration::from_secs(600))
		.build()
		.expect("HTTP client configuration is valid")
}

pub fn upstream(error: impl Display) -> CommonError {
	CommonError::Other(format!("content provider request failed: {error}"))
}

pub fn segment(value: &str) -> Result<&str> {
	let valid = !matches!(value, "" | "." | "..")
		&& value.len() <= 128
		&& value
			.bytes()
			.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
	if valid {
		Ok(value)
	} else {
		Err(CommonError::Invalid(format!("invalid project identifier '{value}'")))
	}
}

pub async fn json<T: DeserializeOwned>(request: RequestBuilder) -> Result<T> {
	checked(request).await?.json().await.map_err(upstream)
}

pub async fn bytes(request: RequestBuilder) -> Result<Vec<u8>> {
	checked(request).await?.bytes().await.map(Vec::from).map_err(upstream)
}

pub async fn download(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
	bytes(client.get(url)).await
}

async fn checked(request: RequestBuilder) -> Result<reqwest::Response> {
	let response = request.send().await.map_err(upstream)?;
	match response.status() {
		StatusCode::NOT_FOUND => Err(CommonError::NotFound("content not found on provider".into())),
		status if status.is_success() => Ok(response),
		status => Err(upstream(format!("HTTP {status}"))),
	}
}

pub fn endpoint(base: &str, segments: &[&str]) -> Result<Url> {
	if let Some(bad) = segments.iter().find(|part| matches!(part.trim(), "" | "." | "..")) {
		return Err(CommonError::Invalid(format!("invalid path segment '{bad}'")));
	}
	let mut url = Url::parse(base).map_err(upstream)?;
	url.path_segments_mut()
		.map_err(|()| upstream("endpoint base cannot carry path segments"))?
		.extend(segments);
	Ok(url)
}
