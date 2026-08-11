#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub const CONSOLE_LOG_FILE: &str = ".obsidian-console.log";
const SESSION_PREFIX: &str = "obsidian-";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenLsEntry {
    pub pid: u32,
    pub name: String,
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn session_name(server_id: u64) -> String {
    format!("{SESSION_PREFIX}{server_id}")
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn server_id_from_session_name(name: &str) -> Option<u64> {
    name.strip_prefix(SESSION_PREFIX)?.parse().ok()
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn parse_screen_ls(output: &str) -> Vec<ScreenLsEntry> {
    output
        .lines()
        .filter_map(|line| {
            let token = line.trim().split_whitespace().next()?;
            let (pid, name) = token.split_once('.')?;
            let pid = pid.parse().ok()?;
            if name.is_empty() {
                return None;
            }
            Some(ScreenLsEntry { pid, name: name.to_string() })
        })
        .collect()
}

#[cfg(target_os = "linux")]
pub use runtime::*;

#[cfg(target_os = "linux")]
mod runtime {
    use super::*;
    use anyhow::{Result, anyhow};
    use std::path::{Path, PathBuf};
    use std::process::Stdio;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
    use tokio::process::Command;
    use tokio::sync::broadcast;

    const TAIL_POLL_INTERVAL: Duration = Duration::from_millis(150);
    const LIVENESS_POLL_INTERVAL: Duration = Duration::from_secs(2);

    pub struct ScreenSession {
        pub name: String,
        pub pid: u32,
        tx: broadcast::Sender<String>,
        stopped: Arc<AtomicBool>,
    }

    pub async fn screen_available() -> bool {
        static AVAILABLE: tokio::sync::OnceCell<bool> = tokio::sync::OnceCell::const_new();
        *AVAILABLE
            .get_or_init(|| async {
                Command::new("screen").arg("-v").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status().await.is_ok()
            })
            .await
    }

    pub async fn list_sessions() -> Vec<ScreenLsEntry> {
        let _ = Command::new("screen").arg("-wipe").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status().await;
        match Command::new("screen").arg("-ls").stdin(Stdio::null()).output().await {
            Ok(out) => parse_screen_ls(&String::from_utf8_lossy(&out.stdout)),
            Err(_) => Vec::new(),
        }
    }

    pub fn is_pid_alive(pid: u32) -> bool {
        Path::new(&format!("/proc/{pid}")).exists()
    }

    impl ScreenSession {
        fn target(&self) -> String {
            format!("{}.{}", self.pid, self.name)
        }

        pub fn subscribe(&self) -> broadcast::Receiver<String> {
            self.tx.subscribe()
        }

        pub async fn spawn(
            server_id: u64,
            program: &str,
            args: &[String],
            working_dir: &Path,
            exit_callback: impl FnOnce(i32) + Send + 'static,
        ) -> Result<Arc<Self>> {
            let name = session_name(server_id);
            let logfile = working_dir.join(CONSOLE_LOG_FILE);
            let _ = tokio::fs::remove_file(&logfile).await;

            let mut child = Command::new("screen")
                .arg("-DmS")
                .arg(&name)
                .arg("-L")
                .arg("-Logfile")
                .arg(&logfile)
                .arg(program)
                .args(args)
                .current_dir(working_dir)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            let pid = child.id().ok_or_else(|| anyhow!("screen session '{name}' exited before a pid was available"))?;

            let session = Self::create(name, pid, logfile, false);

            let target = session.target();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(500)).await;
                let _ = Command::new("screen").args(["-S", &target, "-p", "0", "-X", "logfile", "flush", "0"]).stdin(Stdio::null()).status().await;
            });

            let stopped = session.stopped.clone();
            tokio::spawn(async move {
                let code = child.wait().await.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);
                stopped.store(true, Ordering::Release);
                tokio::time::sleep(TAIL_POLL_INTERVAL * 2).await;
                exit_callback(code);
            });

            Ok(session)
        }

        pub fn reattach(pid: u32, name: String, logfile: PathBuf, exit_callback: impl FnOnce(i32) + Send + 'static) -> Arc<Self> {
            let session = Self::create(name, pid, logfile, true);

            let stopped = session.stopped.clone();
            tokio::spawn(async move {
                while is_pid_alive(pid) {
                    tokio::time::sleep(LIVENESS_POLL_INTERVAL).await;
                }
                stopped.store(true, Ordering::Release);
                tokio::time::sleep(TAIL_POLL_INTERVAL * 2).await;
                exit_callback(0);
            });

            session
        }

        fn create(name: String, pid: u32, logfile: PathBuf, tail_from_end: bool) -> Arc<Self> {
            let (tx, _) = broadcast::channel(1024);
            let stopped = Arc::new(AtomicBool::new(false));
            tokio::spawn(tail_logfile(logfile, tx.clone(), stopped.clone(), tail_from_end));
            Arc::new(Self { name, pid, tx, stopped })
        }

        pub async fn send_input(&self, command: impl Into<String>) -> Result<()> {
            let mut input = command.into();
            input.push('\r');
            let status =
                Command::new("screen").args(["-S", &self.target(), "-p", "0", "-X", "stuff", &input]).stdin(Stdio::null()).status().await?;
            if !status.success() {
                return Err(anyhow!("Failed to send input to screen session '{}'", self.name));
            }
            Ok(())
        }

        pub async fn quit(&self) -> Result<()> {
            Command::new("screen").args(["-S", &self.target(), "-X", "quit"]).stdin(Stdio::null()).status().await?;
            Ok(())
        }
    }

    async fn tail_logfile(path: PathBuf, tx: broadcast::Sender<String>, stopped: Arc<AtomicBool>, from_end: bool) {
        let mut pos: u64 = if from_end { tokio::fs::metadata(&path).await.map(|m| m.len()).unwrap_or(0) } else { 0 };
        let mut partial: Vec<u8> = Vec::new();
        loop {
            let done = stopped.load(Ordering::Acquire);
            if let Ok(mut file) = tokio::fs::File::open(&path).await
                && file.seek(SeekFrom::Start(pos)).await.is_ok()
            {
                let mut buf = Vec::new();
                if file.read_to_end(&mut buf).await.is_ok() && !buf.is_empty() {
                    pos += buf.len() as u64;
                    partial.extend_from_slice(&buf);
                    while let Some(idx) = partial.iter().position(|&b| b == b'\n') {
                        let mut line: Vec<u8> = partial.drain(..=idx).collect();
                        line.pop();
                        if line.last() == Some(&b'\r') {
                            line.pop();
                        }
                        let _ = tx.send(String::from_utf8_lossy(&line).into_owned());
                    }
                }
            }
            if done {
                break;
            }
            tokio::time::sleep(TAIL_POLL_INTERVAL).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_name_is_deterministic() {
        assert_eq!(session_name(7), "obsidian-7");
        assert_eq!(session_name(18446744073709551615), "obsidian-18446744073709551615");
    }

    #[test]
    fn session_name_round_trips() {
        assert_eq!(server_id_from_session_name(&session_name(42)), Some(42));
    }

    #[test]
    fn server_id_rejects_foreign_names() {
        assert_eq!(server_id_from_session_name("irssi"), None);
        assert_eq!(server_id_from_session_name("obsidian-"), None);
        assert_eq!(server_id_from_session_name("obsidian-abc"), None);
        assert_eq!(server_id_from_session_name("obsidian-1x"), None);
    }

    #[test]
    fn parses_typical_screen_ls_output() {
        let output = "There are screens on:\n\t12345.obsidian-3\t(08/10/2026 10:12:33 AM)\t(Detached)\n\t998.irssi\t(Attached)\n2 Sockets in /run/screen/S-admin.\n";
        let entries = parse_screen_ls(output);
        assert_eq!(
            entries,
            vec![
                ScreenLsEntry { pid: 12345, name: "obsidian-3".to_string() },
                ScreenLsEntry { pid: 998, name: "irssi".to_string() },
            ]
        );
    }

    #[test]
    fn parses_single_session_output() {
        let output = "There is a screen on:\n\t4021.obsidian-1\t(Detached)\n1 Socket in /run/screen/S-root.\n";
        assert_eq!(parse_screen_ls(output), vec![ScreenLsEntry { pid: 4021, name: "obsidian-1".to_string() }]);
    }

    #[test]
    fn parses_no_sessions_output() {
        assert!(parse_screen_ls("No Sockets found in /run/screen/S-admin.\n").is_empty());
        assert!(parse_screen_ls("").is_empty());
    }

    #[test]
    fn keeps_names_containing_dots() {
        let output = "\t77.obsidian-2.backup\t(Detached)\n";
        assert_eq!(parse_screen_ls(output), vec![ScreenLsEntry { pid: 77, name: "obsidian-2.backup".to_string() }]);
    }
}
