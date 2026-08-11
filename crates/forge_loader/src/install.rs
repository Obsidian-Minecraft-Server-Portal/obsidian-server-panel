use crate::client::ForgeClient;
use crate::error::{ForgeError, Result};
use crate::models::{ForgeInstallOptions, ForgeInstallResult};
use crate::script_parser;
use futures::StreamExt;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot;
use tokio_interactive::AsynchronousInteractiveProcess;

impl ForgeClient {
    /// Download the Forge installer JAR to the specified directory.
    ///
    /// Returns the path to the downloaded installer JAR.
    pub async fn download_installer(
        &self,
        mc_version: &str,
        forge_version: &str,
        install_dir: &Path,
        progress: Option<&(dyn Fn(u64, u64) + Send + Sync)>,
    ) -> Result<PathBuf> {
        let url = Self::installer_url(mc_version, forge_version);
        let full_version = format!("{}-{}", mc_version, forge_version);
        let installer_name = format!("forge-{}-installer.jar", full_version);
        let installer_path = install_dir.join(&installer_name);

        let response = self.http.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(ForgeError::Api {
                status: response.status().as_u16(),
                message: format!("Failed to download installer from {}", url),
            });
        }

        let total_bytes = response.content_length().unwrap_or(0);
        let mut downloaded: u64 = 0;

        let mut file = tokio::fs::File::create(&installer_path).await?;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(ForgeError::Http)?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            if let Some(cb) = &progress {
                cb(downloaded, total_bytes);
            }
        }

        file.flush().await?;

        Ok(installer_path)
    }

    /// Run the Forge installer (`java -jar installer.jar -installServer`)
    /// in the given directory and parse the resulting start script.
    ///
    /// Uses `tokio_interactive::AsynchronousInteractiveProcess` for process
    /// management. Waits for the installer to complete, then reads and
    /// parses the generated `run.bat` (Windows) or `run.sh` (Unix) to
    /// extract the `@libraries/...` java argument.
    pub async fn run_installer(
        installer_path: &Path,
        install_dir: &Path,
        java_executable: &str,
    ) -> Result<ForgeInstallResult> {
        let install_dir = install_dir.to_path_buf().canonicalize()?;
        let installer_name = installer_path
            .file_name()
            .ok_or_else(|| ForgeError::Other(anyhow::anyhow!("Invalid installer path")))?
            .to_string_lossy()
            .to_string();

        let (tx, rx) = oneshot::channel::<i32>();
        let tx = std::sync::Mutex::new(Some(tx));

        let process = AsynchronousInteractiveProcess::new(java_executable)
            .with_argument("-jar")
            .with_argument(&installer_name)
            .with_argument("-installServer")
            .with_working_directory(&install_dir)
            .process_exit_callback(move |exit_code| {
                if let Some(sender) = tx.lock().unwrap().take() {
                    let _ = sender.send(exit_code);
                }
            })
            .start()
            .await
            .map_err(ForgeError::Other)?;

        let _ = process; // PID not needed; we wait via the channel

        // Wait for the installer to finish
        let exit_code = rx
            .await
            .map_err(|_| ForgeError::Other(anyhow::anyhow!("Installer process channel closed")))?;

        if exit_code != 0 {
            return Err(ForgeError::InstallerFailed { exit_code });
        }

        // Parse the generated start script (modern Forge, 1.17+)
        let script_name = script_parser::start_script_filename();
        let script_path = install_dir.join(script_name);
        if script_path.exists() {
            let script_content = tokio::fs::read_to_string(&script_path).await?;
            return script_parser::parse_start_script(&script_content, exit_code);
        }

        // Legacy Forge (<= 1.16) produces a launchable server jar instead of a run script
        let server_jar = Self::find_legacy_server_jar(&install_dir)?;
        Ok(ForgeInstallResult {
            server_jar,
            java_args: String::new(),
            exit_code,
        })
    }

    /// Locates the universal server jar produced by legacy Forge installers,
    /// e.g. `forge-1.12.2-14.23.5.2860.jar` or `forge-1.12.2-14.23.5.2860-universal.jar`.
    fn find_legacy_server_jar(install_dir: &Path) -> Result<String> {
        std::fs::read_dir(install_dir)
            .map_err(ForgeError::Io)?
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .find(|name| {
                let lower = name.to_lowercase();
                lower.starts_with("forge") && lower.ends_with(".jar") && !lower.contains("installer")
            })
            .ok_or_else(|| ForgeError::ScriptParseError {
                reason: "No run script or forge server jar found after installation".to_string(),
            })
    }

    /// Full installation: download the installer, run it, and parse the
    /// resulting start script.
    pub async fn install_server(
        &self,
        options: ForgeInstallOptions<'_>,
    ) -> Result<ForgeInstallResult> {
        // 1. Download the installer
        let installer_path = self
            .download_installer(
                options.mc_version,
                options.forge_version,
                options.install_dir,
                options
                    .download_progress
                    .as_ref()
                    .map(|f| f.as_ref()),
            )
            .await?;

        // 2. Run the installer and parse the start script
        let result = Self::run_installer(
            &installer_path,
            options.install_dir,
            options.java_executable,
        )
        .await?;

        Ok(result)
    }
}
