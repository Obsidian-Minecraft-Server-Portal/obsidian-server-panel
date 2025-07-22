use std::fs;
use walkdir::WalkDir;

fn main() {
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=./resources/win32");
    }

    #[cfg(target_os = "macos")]
    {
        // macOS-specific icon setup if needed
        println!("cargo:rustc-link-search=native=./resources/icons");
    }

    #[cfg(target_os = "linux")]
    {
        // Linux-specific icon setup if needed
        println!("cargo:rustc-link-search=native=./resources/icons");
    }

    for entry in WalkDir::new("src") {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
    }
    fs::create_dir_all("target/dev-env").expect("failed to create target directory");
    fs::create_dir_all("target/wwwroot").expect("failed to create wwwroot directory");
}
