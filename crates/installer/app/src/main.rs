#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod dialogs;
mod elevation;
mod fonts;
mod handlers;
mod installer;
mod markdown;
mod resources;
mod startup;
mod window;

use anyhow::Result;
use clap::Parser;
use cli::CliArgs;
use installer::{InstallerState, perform_installation};
use log::*;
use slint::ComponentHandle;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = CliArgs::parse();

    // Initialize logging
    pretty_env_logger::env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Debug)
        .init();
    info!("Starting Obsidian Installer");
    debug!("Parsed CLI args: {:?}", args);

    // If headless mode is requested, run without UI
    if args.headless {
        return run_headless(args).await;
    }

    // Otherwise, run GUI mode
    run_gui(args).await
}

/// Run the installer in headless mode (no UI)
async fn run_headless(args: CliArgs) -> Result<()> {
    info!("Running in headless mode");

    // Check TOS acceptance
    if !args.accept_tos {
        anyhow::bail!("Terms of Service must be accepted for headless installation. Use --accept-tos flag.");
    }

    // Check elevation
    if !elevation::is_elevated() {
        info!("Requesting elevation for headless installation...");
        elevation::request_elevation_with_args(&args)?;
        // If we get here, elevation failed
        anyhow::bail!("Failed to obtain administrator privileges");
    }

    info!("Installing to: {}", args.get_install_path());
    info!("Channel: {}", args.channel);
    info!("Install as service: {}", args.service);
    info!("Autostart: {}", args.autostart);

    // Create shared state
    let state = Arc::new(Mutex::new(InstallerState::default()));

    // Perform installation
    perform_installation(
        args.get_install_path(),
        args.service,
        args.channel,
        state.clone(),
    )
    .await?;

    // Handle startup registry if requested
    if args.autostart {
        let exe_path = std::env::current_exe().unwrap_or_default();
        if let Err(e) = startup::add_to_startup(&exe_path) {
            error!("Failed to add to startup: {}", e);
        }
    }

    // Check result
    let final_state = state.lock().unwrap();
    if final_state.success {
        info!("Installation completed successfully!");
        Ok(())
    } else {
        anyhow::bail!("Installation failed: {}", final_state.message);
    }
}

/// Run the installer with GUI
async fn run_gui(args: CliArgs) -> Result<()> {
    info!("Running in GUI mode");

    // Load embedded fonts (WOFF2 fonts need conversion to TTF for Windows GDI)
    // See app/res/fonts/README.md for conversion instructions
    if let Err(e) = fonts::load_embedded_fonts() {
        warn!(
            "Failed to load embedded fonts: {}. Using system fallback.",
            e
        );
    }

    // Create the UI instance
    let ui = App::new()?;

    // Load and set application icon
    if let Some(icon) = resources::load_app_icon() {
        ui.set_app_icon(icon);
    }

    // Check for existing installation before showing UI
    if let Some(existing) = installer::check_existing_installation() {
        info!("Existing installation detected: version {}, path {}", existing.version, existing.install_path.display());
        ui.set_existing_version(existing.version.into());
        ui.set_existing_install_path(existing.install_path.to_string_lossy().to_string().into());
        ui.set_install_path(existing.install_path.to_string_lossy().to_string().into());
        ui.set_current_page(Page::ExistingInstallation);
    }

    // Populate UI from CLI args if provided
    if let Some(path) = &args.install_path {
        ui.set_install_path(path.clone().into());
    }
    ui.set_release_channel(args.channel);
    ui.set_install_as_service(args.service);
    ui.set_start_with_windows(args.autostart);

    // Set page from CLI args if provided (overrides existing installation check)
    let should_start_installation = if let Some(page) = args.page {
        info!("Setting page from CLI args: {}", page);
        ui.set_tos_accepted(true); // Assume TOS accepted when using CLI args
        let target_page = match page {
            0 => Page::Welcome,
            1 => Page::ExistingInstallation,
            2 => Page::TermsOfService,
            3 => Page::Location,
            4 => Page::Installing,
            5 => Page::Complete,
            _ => Page::Welcome,
        };
        ui.set_current_page(target_page);
        page == 4 && !args.uninstall // Start installation if we're on the Installing page and not uninstalling
    } else if args.has_install_params() {
        // If CLI args were provided but no page, skip to location page
        info!("CLI arguments detected, navigating to installation page");
        ui.set_tos_accepted(true); // Assume TOS accepted when using CLI args
        ui.set_current_page(Page::Location);
        false
    } else {
        false
    };

    // Load and parse Terms of Service markdown
    let tos_segments = load_tos_content().await;
    let tos_model = std::rc::Rc::new(slint::VecModel::from(tos_segments));
    ui.set_tos_segments(tos_model.into());

    // Center the window on screen
    let window = ui.window();
    window::center_window(window, 1280.0, 720.0);

    // Setup basic window handlers
    handlers::setup_handlers(&ui);

    // Apply Windows 11 rounded corners after a short delay to ensure window is fully created
    let ui_handle_corners = ui.as_weak();
    slint::Timer::single_shot(std::time::Duration::from_millis(100), move || {
        if let Some(ui) = ui_handle_corners.upgrade() {
            window::apply_rounded_corners(&ui.window());
        }
    });

    // Setup file browser callback
    let ui_handle_browse = ui.as_weak();
    ui.on_browse_folder(move || {
        if let Some(ui) = ui_handle_browse.upgrade()
            && let Some(path) = dialogs::browse_folder()
        {
            ui.set_install_path(path.to_string_lossy().to_string().into());
        }
    });

    // Setup channel versions fetch callback
    let ui_handle_versions = ui.as_weak();
    ui.on_fetch_channel_versions(move || {
        if let Some(ui) = ui_handle_versions.upgrade() {
            ui.set_fetching_versions(true);
            ui.set_release_version("".into());
            ui.set_beta_version("".into());
            ui.set_alpha_version("".into());

            let ui_weak = ui.as_weak();
            tokio::spawn(async move {
                use oim::{InstallationConfig, InstallationManager};
                use std::path::PathBuf;

                // Create a temporary installation manager to fetch versions
                let config = InstallationConfig::new(
                    PathBuf::from("C:\\Program Files\\Obsidian Minecraft Server Panel"),
                    installer::GITHUB_REPO.to_string(),
                    installer::SERVICE_NAME.to_string(),
                );

                let mut manager = InstallationManager::new(config);

                // Fetch versions for all channels
                match manager.get_channel_versions().await {
                    Ok((release, beta, alpha)) => {
                        info!("Received versions - Release: {:?}, Beta: {:?}, Alpha: {:?}",
                                   release, beta, alpha);

                        // Convert to strings before moving into the closure
                        let release_str = release.as_ref().map(|v| v.to_string());
                        let beta_str = beta.as_ref().map(|v| v.to_string());
                        let alpha_str = alpha.as_ref().map(|v| v.to_string());

                        // Use invoke_from_event_loop to update UI on the main thread
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                info!("Updating UI on event loop thread...");

                                if let Some(v) = release_str {
                                    info!("Setting release version to: {}", v);
                                    ui.set_release_version(v.into());
                                }
                                if let Some(v) = beta_str {
                                    info!("Setting beta version to: {}", v);
                                    ui.set_beta_version(v.into());
                                }
                                if let Some(v) = alpha_str {
                                    info!("Setting alpha version to: {}", v);
                                    ui.set_alpha_version(v.into());
                                }
                                ui.set_fetching_versions(false);
                                info!("Finished setting versions in UI");
                            } else {
                                error!("Failed to upgrade UI weak reference in event loop!");
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to fetch channel versions: {}", e);
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_weak.upgrade() {
                                ui.set_fetching_versions(false);
                            }
                        });
                    }
                }
            });
        }
    });

    // Setup installation callback
    let ui_handle_install = ui.as_weak();
    ui.on_start_installation(move || {
        if let Some(ui) = ui_handle_install.upgrade() {
            // Check if we need admin privileges
            if !elevation::is_elevated() {
                warn!("Application is not running with administrator privileges");

                // Build CLI args from current UI state to pass to elevated instance
                let elevation_args = CliArgs {
                    install_path: Some(ui.get_install_path().to_string()),
                    channel: ui.get_release_channel(),
                    service: ui.get_install_as_service(),
                    autostart: ui.get_start_with_windows(),
                    headless: false, // Don't use headless when relaunching from GUI
                    accept_tos: true, // TOS already accepted in UI
                    page: Some(4), // Go directly to Installing page
                    uninstall: false,
                };
                
                debug!("Elevation args: {:?}", elevation_args);

                // Request elevation with args
                match elevation::request_elevation_with_args(&elevation_args) {
                    Ok(_) => {
                        // Successfully requested elevation, this process will exit
                        info!("Elevation requested successfully");
                    }
                    Err(e) => {
                        error!("Failed to request elevation: {}", e);
                        // Show error to user
                        ui.set_current_page(Page::Complete);
                        ui.set_install_success(false);
                        ui.set_complete_message(
                            format!(
                                "Administrator privileges are required to install.\n\nError: {}",
                                e
                            )
                            .into(),
                        );
                    }
                }
                return;
            }

            info!("Running with administrator privileges, proceeding with installation");

            let install_path = ui.get_install_path().to_string();
            let install_as_service = ui.get_install_as_service();
            let start_with_windows = ui.get_start_with_windows();
            let release_channel = ui.get_release_channel();

            // Create shared state
            let state = Arc::new(Mutex::new(InstallerState::default()));
            let state_clone = Arc::clone(&state);
            let ui_weak = ui.as_weak();

            // Spawn installation task
            tokio::spawn(async move {
                // Perform installation
                if let Err(e) = perform_installation(
                    install_path.clone(),
                    install_as_service,
                    release_channel,
                    state_clone.clone(),
                )
                .await
                {
                    error!("Installation error: {}", e);
                    let mut s = state_clone.lock().unwrap();
                    s.success = false;
                    s.completed = true;
                    s.has_error = true;
                    s.error_message = format!("{}", e);
                    s.message = format!("Installation failed: {}", e);
                }

                // Handle startup registry if requested
                if start_with_windows {
                    let exe_path = std::env::current_exe().unwrap_or_default();
                    if let Err(e) = startup::add_to_startup(&exe_path) {
                        error!("Failed to add to startup: {}", e);
                    }
                }

                // Wait a moment for final state update
                tokio::time::sleep(Duration::from_millis(500)).await;

                // Update UI to complete page on event loop thread
                let (success, message) = {
                    let s = state_clone.lock().unwrap();
                    (s.success, s.message.clone())
                };

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_install_success(success);
                        ui.set_complete_message(message.into());
                        ui.set_current_page(Page::Complete);
                    }
                });
            });

            // Start progress monitoring
            let state_monitor = Arc::clone(&state);
            let ui_weak_monitor = ui.as_weak();

            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(100)).await;

                    // Read state outside the event loop
                    let (status, progress, has_error, error_message, completed) = {
                        let s = state_monitor.lock().unwrap();
                        (
                            s.status.clone(),
                            s.progress,
                            s.has_error,
                            s.error_message.clone(),
                            s.completed,
                        )
                    };

                    // Update UI on event loop thread
                    let ui_weak_clone = ui_weak_monitor.clone();
                    let update_result = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak_clone.upgrade() {
                            ui.set_install_status(status.into());
                            ui.set_install_progress(progress);
                            ui.set_install_has_error(has_error);
                            ui.set_install_error_message(error_message.into());
                        }
                    });

                    if completed || update_result.is_err() {
                        break;
                    }
                }
            });
        }
    });

    // Setup launch app callback
    let ui_handle_launch = ui.as_weak();
    ui.on_launch_app(move || {
        if let Some(ui) = ui_handle_launch.upgrade() {
            let install_path = ui.get_install_path().to_string();
            launch_application(&install_path);
        }
    });

    // Setup repair callback
    let ui_handle_repair = ui.as_weak();
    ui.on_repair_installation(move || {
        if let Some(ui) = ui_handle_repair.upgrade() {
            info!("Repair installation requested");

            // Check if we need admin privileges
            if !elevation::is_elevated() {
                warn!("Application is not running with administrator privileges");

                // Build CLI args to pass to elevated instance
                let elevation_args = CliArgs {
                    install_path: Some(ui.get_existing_install_path().to_string()),
                    channel: ui.get_release_channel(),
                    service: ui.get_install_as_service(),
                    autostart: ui.get_start_with_windows(),
                    headless: false,
                    accept_tos: true,
                    page: Some(4), // Go directly to Installing page
                    uninstall: false,
                };

                debug!("Requesting elevation for repair: {:?}", elevation_args);

                // Request elevation with args
                match elevation::request_elevation_with_args(&elevation_args) {
                    Ok(_) => {
                        // Successfully requested elevation, this process will exit
                        info!("Elevation requested successfully for repair");
                    }
                    Err(e) => {
                        error!("Failed to request elevation: {}", e);
                        // Show error to user
                        ui.set_current_page(Page::Complete);
                        ui.set_install_success(false);
                        ui.set_complete_message(
                            format!(
                                "Administrator privileges are required to repair the installation.\\n\\nError: {}",
                                e
                            )
                            .into(),
                        );
                    }
                }
                return;
            }

            info!("Running with administrator privileges, proceeding with repair");

            // Navigate to Installing page
            ui.set_current_page(Page::Installing);

            let install_path = ui.get_install_path().to_string();
            let release_channel = ui.get_release_channel();

            // Create shared state
            let state = Arc::new(Mutex::new(installer::InstallerState::default()));
            let state_clone = Arc::clone(&state);
            let ui_weak = ui.as_weak();

            // Spawn repair task
            tokio::spawn(async move {
                if let Err(e) = installer::perform_repair(install_path, release_channel, state_clone.clone()).await {
                    error!("Repair error: {}", e);
                    let mut s = state_clone.lock().unwrap();
                    s.success = false;
                    s.completed = true;
                    s.has_error = true;
                    s.error_message = format!("{}", e);
                    s.message = format!("Repair failed: {}", e);
                }

                // Wait a moment for final state update
                tokio::time::sleep(Duration::from_millis(500)).await;

                // Update UI to complete page on event loop thread
                let (success, message) = {
                    let s = state_clone.lock().unwrap();
                    (s.success, s.message.clone())
                };

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        ui.set_install_success(success);
                        ui.set_complete_message(message.into());
                        ui.set_current_page(Page::Complete);
                    }
                });
            });

            // Start progress monitoring
            let state_monitor = Arc::clone(&state);
            let ui_weak_monitor = ui.as_weak();

            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(100)).await;

                    // Read state outside the event loop
                    let (status, progress, has_error, error_message, completed) = {
                        let s = state_monitor.lock().unwrap();
                        (
                            s.status.clone(),
                            s.progress,
                            s.has_error,
                            s.error_message.clone(),
                            s.completed,
                        )
                    };

                    // Update UI on event loop thread
                    let ui_weak_clone = ui_weak_monitor.clone();
                    let update_result = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak_clone.upgrade() {
                            ui.set_install_status(status.into());
                            ui.set_install_progress(progress);
                            ui.set_install_has_error(has_error);
                            ui.set_install_error_message(error_message.into());
                        }
                    });

                    if completed || update_result.is_err() {
                        break;
                    }
                }
            });
        }
    });

    // Setup uninstall callback
    let ui_handle_uninstall = ui.as_weak();
    ui.on_uninstall_installation(move || {
        if let Some(ui) = ui_handle_uninstall.upgrade() {
            info!("Uninstall requested");

            // Check if we need admin privileges
            if !elevation::is_elevated() {
                warn!("Application is not running with administrator privileges");

                // Build CLI args to pass to elevated instance
                let elevation_args = CliArgs {
                    install_path: Some(ui.get_existing_install_path().to_string()),
                    channel: ui.get_release_channel(),
                    service: ui.get_install_as_service(),
                    autostart: ui.get_start_with_windows(),
                    headless: false,
                    accept_tos: true,
                    page: Some(4), // Go directly to Installing page
                    uninstall: true, // Flag to indicate uninstall mode
                };

                debug!("Requesting elevation for uninstall: {:?}", elevation_args);

                // Request elevation with args
                match elevation::request_elevation_with_args(&elevation_args) {
                    Ok(_) => {
                        // Successfully requested elevation, this process will exit
                        info!("Elevation requested successfully for uninstall");
                    }
                    Err(e) => {
                        error!("Failed to request elevation: {}", e);
                        // Show error to user
                        ui.set_current_page(Page::Complete);
                        ui.set_install_success(false);
                        ui.set_complete_message(
                            format!(
                                "Administrator privileges are required to uninstall.\\n\\nError: {}",
                                e
                            )
                            .into(),
                        );
                    }
                }
                return;
            }

            info!("Running with administrator privileges, proceeding with uninstall");

            // Navigate to Installing page
            ui.set_current_page(Page::Installing);

            // Create shared state
            let state = Arc::new(Mutex::new(installer::InstallerState::default()));
            let state_clone = Arc::clone(&state);
            let ui_weak = ui.as_weak();

            // Spawn uninstall task
            tokio::spawn(async move {
                if let Err(e) = installer::perform_uninstall(state_clone.clone()).await {
                    error!("Uninstall error: {}", e);
                    let mut s = state_clone.lock().unwrap();
                    s.success = false;
                    s.completed = true;
                    s.has_error = true;
                    s.error_message = format!("{}", e);
                    s.message = format!("Uninstall failed: {}", e);
                }

                // Wait a moment for final state update
                tokio::time::sleep(Duration::from_millis(500)).await;

                // Check if uninstall was successful
                let (success, message) = {
                    let s = state_clone.lock().unwrap();
                    (s.success, s.message.clone())
                };

                if success {
                    // For successful uninstall, just exit the application
                    info!("Uninstall completed successfully, exiting application");
                    std::process::exit(0);
                } else {
                    // For failed uninstall, show error on complete page
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            ui.set_install_success(false);
                            ui.set_complete_message(message.into());
                            ui.set_current_page(Page::Complete);
                        }
                    });
                }
            });

            // Start progress monitoring
            let state_monitor = Arc::clone(&state);
            let ui_weak_monitor = ui.as_weak();

            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_millis(100)).await;

                    // Read state outside the event loop
                    let (status, progress, has_error, error_message, completed) = {
                        let s = state_monitor.lock().unwrap();
                        (
                            s.status.clone(),
                            s.progress,
                            s.has_error,
                            s.error_message.clone(),
                            s.completed,
                        )
                    };

                    // Update UI on event loop thread
                    let ui_weak_clone = ui_weak_monitor.clone();
                    let update_result = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak_clone.upgrade() {
                            ui.set_install_status(status.into());
                            ui.set_install_progress(progress);
                            ui.set_install_has_error(has_error);
                            ui.set_install_error_message(error_message.into());
                        }
                    });

                    if completed || update_result.is_err() {
                        break;
                    }
                }
            });
        }
    });

    // Setup cancel existing installation callback
    let ui_handle_cancel = ui.as_weak();
    ui.on_cancel_existing_installation(move || {
        if let Some(_ui) = ui_handle_cancel.upgrade() {
            info!("Cancelled existing installation prompt");
            // Just exit the application
            std::process::exit(0);
        }
    });

    // If we should start installation (elevated instance on Installing page), trigger it
    if should_start_installation {
        info!("Auto-starting installation from elevated instance");
        ui.invoke_start_installation();
    }

    // If uninstall flag is set, trigger uninstall
    if args.uninstall {
        info!("Auto-starting uninstall from elevated instance");
        ui.invoke_uninstall_installation();
    }

    // Run the application
    ui.run()?;
    Ok(())
}

/// Loads the Terms of Service content from GitHub
async fn load_tos_content() -> Vec<TextSegment> {
    let result = reqwest::get("https://raw.githubusercontent.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel/refs/heads/main/terms-of-service.md").await;

    let markdown = if let Ok(response) = result
        && response.status().is_success()
    {
        let text = response.text().await.unwrap_or_default();
        let lines: Vec<&str> = text.lines().collect();
        lines[1..].join("\n")
    } else {
        "Failed to load the Terms of Service, Please visit our [github](https://github.com/Obsidian-Minecraft-Server-Portal/obsidian-server-panel.git) for the terms of service".to_string()
    };

    markdown::parse_markdown_to_segments(&markdown)
}

/// Launches the installed application and exits the installer
fn launch_application(install_path: &str) {
    let install_dir = PathBuf::from(install_path);

    // Try to find and launch the executable
    if let Ok(entries) = fs::read_dir(&install_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("exe") {
                info!("Launching application: {}", path.display());

                #[cfg(target_os = "windows")]
                {
                    use std::process::Command;
                    match Command::new("explorer.exe").arg(&path).current_dir(&install_dir).spawn() {
                        Ok(_) => {
                            info!("Application launched successfully, exiting installer");
                            std::process::exit(0);
                        }
                        Err(e) => {
                            error!("Failed to launch application: {}", e);
                        }
                    }
                }

                #[cfg(not(target_os = "windows"))]
                {
                    // Non-Windows platforms
                    info!("Launch not implemented for this platform");
                }

                break;
            }
        }
    }
}
