use obsidian_common::{InstanceId, Result};
use obsidian_maps::detect;
use obsidian_net::{Protocol, UpnpStatus};

use crate::manager::InstanceManager;

impl InstanceManager {
	pub async fn set_map_upnp(&self, id: InstanceId, enabled: bool) -> Result<()> {
		sqlx::query("UPDATE instances SET map_upnp_enabled = ? WHERE id = ?")
			.bind(enabled)
			.bind(id)
			.execute(self.db.pool())
			.await?;
		self.sync_map_upnp(id).await;
		Ok(())
	}

	pub async fn map_upnp_enabled(&self, id: InstanceId) -> Result<bool> {
		let enabled: Option<bool> = sqlx::query_scalar("SELECT map_upnp_enabled FROM instances WHERE id = ?")
			.bind(id)
			.fetch_optional(self.db.pool())
			.await?;
		Ok(enabled.unwrap_or(false))
	}

	pub async fn map_upnp_status(&self, id: InstanceId) -> Result<Vec<(u16, UpnpStatus)>> {
		let ports = self.live().get(&id).map(|live| live.map_upnp_ports.clone()).unwrap_or_default();
		let mut statuses = Vec::with_capacity(ports.len());
		for port in ports {
			statuses.push((port, self.upnp.status(port).await?));
		}
		Ok(statuses)
	}

	pub(crate) async fn sync_map_upnp(&self, id: InstanceId) {
		let wanted = self.wanted_map_ports(id).await;
		let current = self.live().get(&id).map(|live| live.map_upnp_ports.clone()).unwrap_or_default();
		if wanted == current {
			return;
		}
		self.unmap_map_upnp(id).await;
		let Ok(instance) = self.get(id).await else { return };
		let description = format!("Obsidian {} map", instance.name);
		let mut mapped = Vec::with_capacity(wanted.len());
		for port in wanted {
			match self.upnp.map(port, Protocol::Tcp, &description).await {
				Ok(()) => mapped.push(port),
				Err(error) => tracing::warn!(%error, instance = id, port, "UPnP map port mapping failed"),
			}
		}
		self.live().entry(id).or_default().map_upnp_ports = mapped;
	}

	pub(crate) async fn unmap_map_upnp(&self, id: InstanceId) {
		let ports = self.live().get_mut(&id).map(|live| std::mem::take(&mut live.map_upnp_ports)).unwrap_or_default();
		for port in ports {
			if let Err(error) = self.upnp.unmap(port).await {
				tracing::warn!(%error, instance = id, port, "failed to remove map UPnP mapping");
			}
		}
	}

	async fn wanted_map_ports(&self, id: InstanceId) -> Vec<u16> {
		if !self.is_active_instance(id) || !self.map_upnp_enabled(id).await.unwrap_or(false) {
			return Vec::new();
		}
		match detect(&self.paths.instance_dir(id)).await {
			Ok(maps) => maps.into_iter().filter(|map| map.settings.enabled).map(|map| map.settings.port).collect(),
			Err(error) => {
				tracing::warn!(%error, instance = id, "could not read the web maps for UPnP");
				Vec::new()
			}
		}
	}
}
