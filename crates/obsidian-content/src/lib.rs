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
pub use manager::{ContentManager, InstalledWorld, ServerResourcePack};
pub use model::{
	Category, CategoryGroup, ContentUpdate, Facets, InstallRequest, InstalledContent, ModpackRequest, Project, ProjectVersion,
	SearchPage, SearchQuery, Side, VersionFilter,
};
pub use platform::{ContentPlatform, Kind, Platform};

pub const CURSEFORGE_API_KEY: Option<&str> = option_env!("CURSEFORGE_API_KEY");
