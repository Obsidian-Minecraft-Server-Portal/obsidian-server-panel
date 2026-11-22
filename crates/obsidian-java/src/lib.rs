mod detect;
mod manager;
mod piston;
mod probe;
mod runtime;

pub use manager::JavaManager;
pub use runtime::{JavaRequirement, JavaRuntime, JavaSource};

pub const COMPONENTS: [&str; 7] = [
	"java-runtime-alpha",
	"java-runtime-beta",
	"java-runtime-gamma",
	"java-runtime-delta",
	"java-runtime-gamma-snapshot",
	"java-runtime-epsilon",
	"jre-legacy",
];

pub fn is_component(name: &str) -> bool {
	COMPONENTS.contains(&name)
}
