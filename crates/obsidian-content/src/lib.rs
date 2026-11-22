mod archive;
mod http;
mod icon;
mod manager;
mod model;
mod modpack;
mod platform;
mod platforms;
#[cfg(test)]
mod tests;

pub use icon::mime as icon_mime;
pub use manager::ContentManager;
pub use model::{
	ContentUpdate, InstallRequest, InstalledContent, ModpackRequest, Project, ProjectVersion, SearchPage, SearchQuery,
	VersionFilter,
};
pub use platform::{ContentPlatform, Kind, Platform};

pub const CURSEFORGE_API_KEY: Option<&str> = option_env!("CURSEFORGE_API_KEY");
