use std::fs;

const WWWROOT: &str = "target/wwwroot";

fn main() {
	fs::create_dir_all(WWWROOT).expect("failed to create target/wwwroot");
	println!("cargo:rerun-if-changed=build.rs");
	println!("cargo:rerun-if-changed={WWWROOT}");
	println!("cargo:rustc-env=BUILD={}", chrono::Local::now().format("%Y%m%d.%H%M%S"));
}
