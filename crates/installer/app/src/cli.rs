use clap::Parser;

/// Obsidian Minecraft Server Panel Installer
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Installation path
    #[arg(long, value_name = "PATH")]
    pub install_path: Option<String>,

    /// Release channel: 0=Release (Stable), 1=Beta, 2=Alpha
    #[arg(long, value_name = "CHANNEL", default_value = "0")]
    pub channel: i32,

    /// Install as Windows Service
    #[arg(long)]
    pub service: bool,

    /// Start with Windows
    #[arg(long, default_value = "false")]
    pub autostart: bool,

    /// Run installation without GUI (headless mode)
    #[arg(long)]
    pub headless: bool,

    /// Accept Terms of Service (required for headless mode)
    #[arg(long)]
    pub accept_tos: bool,

    /// Initial page to display: 0=Welcome, 1=ExistingInstallation, 2=TermsOfService, 3=Location, 4=Installing, 5=Complete
    #[arg(long, value_name = "PAGE")]
    pub page: Option<i32>,

    /// Uninstall mode (used internally when relaunching for uninstall)
    #[arg(long)]
    pub uninstall: bool,
}

impl CliArgs {
    /// Check if any installation parameters were provided
    pub fn has_install_params(&self) -> bool {
        self.install_path.is_some() || self.channel != 0 || self.service || self.autostart
    }

    /// Build command line arguments string for relaunching with elevation
    pub fn to_args_string(&self) -> String {
        let mut args = Vec::new();

        if let Some(path) = &self.install_path {
            // Properly escape the path for Windows command line
            // Use backslashes to escape quotes if the path contains spaces
            if path.contains(' ') {
                args.push(format!("--install-path \"{}\"", path.replace("\"", "\\\"")));
            } else {
                args.push(format!("--install-path {}", path));
            }
        }

        if self.channel != 0 {
            args.push(format!("--channel {}", self.channel));
        }

        if self.service {
            args.push("--service".to_string());
        }

        if self.autostart {
            args.push("--autostart".to_string());
        }

        if self.headless {
            args.push("--headless".to_string());
        }

        if self.accept_tos {
            args.push("--accept-tos".to_string());
        }

        if let Some(page) = self.page {
            args.push(format!("--page {}", page));
        }

        if self.uninstall {
            args.push("--uninstall".to_string());
        }

        args.join(" ")
    }

    /// Get the install path or use the default
    pub fn get_install_path(&self) -> String {
        self.install_path
            .clone()
            .unwrap_or_else(|| "C:\\Program Files\\Obsidian Minecraft Server Panel".to_string())
    }
}
