use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::Path;

use java_properties::{PropertiesError, PropertiesWriter};
use obsidian_common::{CommonError, InstanceId, Result};

use crate::manager::InstanceManager;
use crate::model::UpdateInstance;

const FILE: &str = "server.properties";
const PORT_KEY: &str = "server-port";
const HEADER: &str = "Minecraft server properties";

fn invalid(error: PropertiesError) -> CommonError {
	CommonError::Other(format!("server.properties: {error}"))
}

pub(crate) async fn read(dir: &Path) -> Result<BTreeMap<String, String>> {
	let bytes = match tokio::fs::read(dir.join(FILE)).await {
		Ok(bytes) => bytes,
		Err(error) if error.kind() == ErrorKind::NotFound => return Ok(BTreeMap::new()),
		Err(error) => return Err(error.into()),
	};
	Ok(java_properties::read(bytes.as_slice()).map_err(invalid)?.into_iter().collect())
}

pub(crate) async fn write(dir: &Path, properties: &BTreeMap<String, String>) -> Result<()> {
	let mut buffer = Vec::new();
	let mut writer = PropertiesWriter::new(&mut buffer);
	writer.write_comment(HEADER).map_err(invalid)?;
	for (key, value) in properties {
		writer.write(key, value).map_err(invalid)?;
	}
	writer.finish().map_err(invalid)?;
	drop(writer);
	tokio::fs::write(dir.join(FILE), buffer).await?;
	Ok(())
}

pub(crate) async fn sync_port(dir: &Path, port: u16) -> Result<()> {
	let mut properties = read(dir).await?;
	properties.insert(PORT_KEY.to_owned(), port.to_string());
	write(dir, &properties).await
}

impl InstanceManager {
	pub async fn properties(&self, id: InstanceId) -> Result<BTreeMap<String, String>> {
		let instance = self.get(id).await?;
		read(&instance.directory).await
	}

	pub async fn set_properties(&self, id: InstanceId, mut properties: BTreeMap<String, String>) -> Result<()> {
		let instance = self.get(id).await?;
		let requested_port = properties.get(PORT_KEY).and_then(|value| value.trim().parse::<u16>().ok());
		if let Some(port) = requested_port.filter(|port| *port != instance.port) {
			self.update(id, UpdateInstance { port: Some(port), ..Default::default() }).await?;
		}
		let instance = self.get(id).await?;
		properties.insert(PORT_KEY.to_owned(), instance.port.to_string());
		write(&instance.directory, &properties).await
	}
}
