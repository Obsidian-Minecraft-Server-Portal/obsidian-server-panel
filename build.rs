use std::fs;
use walkdir::WalkDir;
use chrono::Timelike;

fn main() {
	for entry in WalkDir::new("src") {
		let entry = entry.unwrap();
		if entry.file_type().is_file() {
			println!("cargo:rerun-if-changed={}", entry.path().display());
		}
	}
	fs::create_dir_all("target/dev-env").expect("failed to create target directory");
	fs::create_dir_all("target/wwwroot").expect("failed to create wwwroot directory");
	generate_build_number();
}

fn generate_build_number() {
	let seconds_past_in_day = chrono::Local::now().num_seconds_from_midnight();
	let build = chrono::Local::now().format("%Y%m%d").to_string();
	let build = format!("{}.{:06}", build, seconds_past_in_day);
	println!("cargo:rustc-env=BUILD={build}");
}