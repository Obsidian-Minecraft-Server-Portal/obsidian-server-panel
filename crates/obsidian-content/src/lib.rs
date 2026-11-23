mod archive;
mod http;
mod icon;
mod manager;
mod model;
mod modpack;
mod metadata;
mod platform;
mod platforms;
mod render;
mod serverpack;
#[cfg(test)]
mod tests;

pub use icon::mime as icon_mime;
pub use manager::{
	ContentManager, ContentRelations, ContentSource, DependencyEntry, DependentEntry, InstalledWorld, QuickAdded, ServerResourcePack,
};
pub use model::{
	Category, CategoryGroup, ContentUpdate, DependencyKind, Facets, GalleryImage, InstallRequest, InstalledContent, ModpackRequest, PackSource,
	Project, ProjectDetail, ProjectLinks, ProjectVersion, SearchPage, SearchQuery, Side, VersionDependency, VersionFilter,
};
pub use platform::{ContentPlatform, Kind, Platform};
pub use serverpack::{
	AnalysisResult, AnalyzeRequest, AnalyzedMod, DetectedSide, ExportRequest, FolderChoice, FolderDecision, Lean, ModDecision,
};

pub const CURSEFORGE_API_KEY: Option<&str> = option_env!("CURSEFORGE_API_KEY");
