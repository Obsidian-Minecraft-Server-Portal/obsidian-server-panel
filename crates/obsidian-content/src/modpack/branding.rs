use obsidian_instances::InstanceManager;

use crate::http;
use crate::manager::ContentManager;
use crate::platform::ContentPlatform;

impl ContentManager {
	pub(crate) async fn apply_branding(
		&self,
		instances: &InstanceManager,
		instance_id: i64,
		provider: &dyn ContentPlatform,
		project_id: &str,
	) -> Vec<String> {
		let project = match provider.project(project_id).await {
			Ok(project) => project,
			Err(error) => return vec![format!("the modpack's icon and banner were not copied: {error}")],
		};
		let mut warnings = Vec::new();
		if let Some(url) = project.icon_url.as_deref() {
			let outcome = match http::download(&self.http, url).await {
				Ok(bytes) => instances.set_icon(instance_id, bytes).await,
				Err(error) => Err(error),
			};
			if let Err(error) = outcome {
				warnings.push(format!("the server icon was not set: {error}"));
			}
		}
		if let Some(url) = project.banner_url.as_deref() {
			let outcome = match http::download(&self.http, url).await {
				Ok(bytes) => instances.set_banner(instance_id, bytes).await,
				Err(error) => Err(error),
			};
			if let Err(error) = outcome {
				warnings.push(format!("the server banner was not set: {error}"));
			}
		}
		warnings
	}
}
