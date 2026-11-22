#[cfg(feature = "service")]
use std::path::PathBuf;

use clap::Subcommand;
use color_eyre::Result;
use color_eyre::eyre::bail;
#[cfg(not(all(feature = "service", windows)))]
use tokio::sync::oneshot;

#[cfg(feature = "service")]
const LABEL: &str = "com.obsidian.server-panel";
#[cfg(feature = "service")]
const DATA_DIR_VAR: &str = "OBSIDIAN_DATA_DIR";

#[derive(Debug, Clone, Copy, Subcommand)]
pub enum ServiceAction {
	/// Register the panel as an OS service that starts on boot
	Install,
	/// Remove the OS service registration
	Uninstall,
	/// Start the installed OS service
	Start,
	/// Stop the running OS service
	Stop,
	/// Entry point used by the Windows service manager
	#[command(hide = true)]
	Run,
}

#[cfg(feature = "service")]
pub fn run(action: ServiceAction) -> Result<()> {
	use service_manager::{
		ServiceLevel, ServiceManager, ServiceStartCtx, ServiceStopCtx, ServiceUninstallCtx,
	};

	let label = LABEL.parse()?;
	let mut manager = <dyn ServiceManager>::native()?;
	manager.set_level(ServiceLevel::System)?;
	match action {
		ServiceAction::Run => bail!("`service run` is started by the Windows service manager"),
		ServiceAction::Install => {
			manager.install(install_context(label)?)?;
			#[cfg(windows)]
			windows::configure_recovery()?;
		}
		ServiceAction::Uninstall => manager.uninstall(ServiceUninstallCtx { label })?,
		ServiceAction::Start => manager.start(ServiceStartCtx { label })?,
		ServiceAction::Stop => {
			manager.stop(ServiceStopCtx { label })?;
			#[cfg(windows)]
			windows::wait_until_stopped()?;
		}
	}
	Ok(())
}

#[cfg(not(feature = "service"))]
pub fn run(action: ServiceAction) -> Result<()> {
	bail!("service {action:?} is not available in this build")
}

#[cfg(all(feature = "service", windows))]
pub use windows::dispatch;
#[cfg(all(feature = "service", feature = "updater", windows))]
pub use windows::is_dispatched as is_windows_service;

#[cfg(not(all(feature = "service", windows)))]
pub fn dispatch<F>(serve: F) -> Result<()>
where
	F: FnOnce(oneshot::Receiver<()>) -> Result<()> + Send + 'static,
{
	drop(serve);
	bail!("the service entry point is only available in Windows builds with the service feature")
}

#[cfg(all(feature = "updater", not(all(feature = "service", windows))))]
pub fn is_windows_service() -> bool {
	false
}

#[cfg(feature = "service")]
pub fn service_root() -> Result<PathBuf> {
	let dir = std::env::var_os(DATA_DIR_VAR).map_or_else(system_data_dir, PathBuf::from);
	Ok(std::path::absolute(dir)?)
}

#[cfg(feature = "service")]
fn system_data_dir() -> PathBuf {
	if cfg!(windows) {
		std::env::var_os("ProgramData")
			.map_or_else(|| PathBuf::from(r"C:\ProgramData"), PathBuf::from)
			.join("Obsidian")
	} else if cfg!(target_os = "macos") {
		PathBuf::from("/Library/Application Support/Obsidian")
	} else {
		PathBuf::from("/var/lib/obsidian")
	}
}

#[cfg(feature = "service")]
fn install_context(
	label: service_manager::ServiceLabel,
) -> Result<service_manager::ServiceInstallCtx> {
	use service_manager::{RestartPolicy, ServiceInstallCtx};

	let root = service_root()?;
	std::fs::create_dir_all(&root)?;
	let program = std::env::current_exe()?;
	let args = if cfg!(windows) { vec!["service".into(), "run".into()] } else { vec!["serve".into(), "--headless".into()] };
	Ok(ServiceInstallCtx {
		label,
		contents: unit_contents(&program, &root),
		program,
		args,
		username: None,
		environment: Some(vec![(DATA_DIR_VAR.into(), root.display().to_string())]),
		working_directory: Some(root),
		autostart: true,
		restart_policy: RestartPolicy::OnFailure {
			delay_secs: Some(5),
			max_retries: None,
			reset_after_secs: None,
		},
	})
}

/// `KillMode=process` stops only the panel on stop or restart, so detached Minecraft runners survive.
#[cfg(all(feature = "service", target_os = "linux"))]
fn unit_contents(program: &std::path::Path, root: &std::path::Path) -> Option<String> {
	let (exe, dir) = (program.display(), root.display());
	Some(format!(
		"[Unit]
Description=Obsidian Minecraft server panel
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=\"{exe}\" serve --headless
WorkingDirectory={dir}
Environment={DATA_DIR_VAR}={dir}
Restart=on-failure
RestartSec=5
KillMode=process
TimeoutStopSec=60

[Install]
WantedBy=multi-user.target
"
	))
}

#[cfg(all(feature = "service", not(target_os = "linux")))]
fn unit_contents(_: &std::path::Path, _: &std::path::Path) -> Option<String> {
	None
}

#[cfg(all(feature = "service", windows))]
mod windows {
	use std::ffi::OsString;
	use std::sync::atomic::{AtomicBool, Ordering};
	use std::sync::{Mutex, OnceLock, PoisonError};
	use std::time::{Duration, Instant};

	use color_eyre::Result;
	use color_eyre::eyre::{bail, eyre};
	use tokio::sync::oneshot;
	use windows_service::service::{
		Service, ServiceAccess, ServiceAction, ServiceActionType, ServiceControl,
		ServiceControlAccept, ServiceExitCode, ServiceFailureActions, ServiceFailureResetPeriod,
		ServiceState, ServiceStatus, ServiceType,
	};
	use windows_service::service_control_handler::{
		self, ServiceControlHandlerResult, ServiceStatusHandle,
	};
	use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
	use windows_service::{define_windows_service, service_dispatcher};

	use super::LABEL;

	type Serve = Box<dyn FnOnce(oneshot::Receiver<()>) -> Result<()> + Send>;

	const RESTART_DELAY: Duration = Duration::from_secs(5);
	const FAILURE_RESET: Duration = Duration::from_secs(24 * 60 * 60);
	const STOP_WAIT_HINT: Duration = Duration::from_secs(60);
	const STOP_TIMEOUT: Duration = Duration::from_secs(90);
	const POLL_INTERVAL: Duration = Duration::from_millis(500);

	static SERVE: Mutex<Option<Serve>> = Mutex::new(None);
	static STATUS: OnceLock<ServiceStatusHandle> = OnceLock::new();
	static DISPATCHED: AtomicBool = AtomicBool::new(false);

	define_windows_service!(ffi_service_main, service_main);

	/// Hands the panel to the Windows service dispatcher; the receiver resolves when the service is asked to stop.
	pub fn dispatch<F>(serve: F) -> Result<()>
	where
		F: FnOnce(oneshot::Receiver<()>) -> Result<()> + Send + 'static,
	{
		*SERVE.lock().unwrap_or_else(PoisonError::into_inner) = Some(Box::new(serve));
		service_dispatcher::start(LABEL, ffi_service_main)?;
		Ok(())
	}

	#[cfg(feature = "updater")]
	pub fn is_dispatched() -> bool {
		DISPATCHED.load(Ordering::Relaxed)
	}

	fn service_main(_: Vec<OsString>) {
		DISPATCHED.store(true, Ordering::Relaxed);
		if let Err(error) = run_service() {
			tracing::error!("windows service failed: {error:#}");
		}
	}

	fn run_service() -> Result<()> {
		let serve = SERVE
			.lock()
			.unwrap_or_else(PoisonError::into_inner)
			.take()
			.ok_or_else(|| eyre!("the service started without a panel entry point"))?;
		let (shutdown_tx, shutdown_rx) = oneshot::channel();
		let mut shutdown_tx = Some(shutdown_tx);
		let handle = service_control_handler::register(LABEL, move |control| match control {
			ServiceControl::Stop | ServiceControl::Shutdown => {
				if let Some(&status) = STATUS.get() {
					let _ = report(status, ServiceState::StopPending, STOP_WAIT_HINT, 0);
				}
				if let Some(tx) = shutdown_tx.take() {
					let _ = tx.send(());
				}
				ServiceControlHandlerResult::NoError
			}
			ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
			_ => ServiceControlHandlerResult::NotImplemented,
		})?;
		let status = *STATUS.get_or_init(|| handle);
		let outcome = prepare().and_then(|()| {
			report(status, ServiceState::Running, Duration::ZERO, 0)?;
			serve(shutdown_rx)
		});
		report(status, ServiceState::Stopped, Duration::ZERO, u32::from(outcome.is_err()))?;
		outcome
	}

	fn prepare() -> Result<()> {
		let root = super::service_root()?;
		std::fs::create_dir_all(&root)?;
		std::env::set_current_dir(root)?;
		Ok(())
	}

	fn report(
		handle: ServiceStatusHandle,
		state: ServiceState,
		wait_hint: Duration,
		exit_code: u32,
	) -> Result<()> {
		let controls_accepted = match state {
			ServiceState::Running => ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
			_ => ServiceControlAccept::empty(),
		};
		handle.set_service_status(ServiceStatus {
			service_type: ServiceType::OWN_PROCESS,
			current_state: state,
			controls_accepted,
			exit_code: ServiceExitCode::Win32(exit_code),
			checkpoint: 0,
			wait_hint,
			process_id: None,
		})?;
		Ok(())
	}

	pub fn configure_recovery() -> Result<()> {
		let service = open(ServiceAccess::CHANGE_CONFIG | ServiceAccess::START)?;
		let restart = ServiceAction {
			action_type: ServiceActionType::Restart,
			delay: RESTART_DELAY,
		};
		service.update_failure_actions(ServiceFailureActions {
			reset_period: ServiceFailureResetPeriod::After(FAILURE_RESET),
			reboot_msg: None,
			command: None,
			actions: Some(vec![restart; 3]),
		})?;
		service.set_failure_actions_on_non_crash_failures(true)?;
		Ok(())
	}

	pub fn wait_until_stopped() -> Result<()> {
		let service = open(ServiceAccess::QUERY_STATUS)?;
		let deadline = Instant::now() + STOP_TIMEOUT;
		while service.query_status()?.current_state != ServiceState::Stopped {
			if Instant::now() >= deadline {
				bail!("timed out waiting for the service to stop");
			}
			std::thread::sleep(POLL_INTERVAL);
		}
		Ok(())
	}

	fn open(access: ServiceAccess) -> Result<Service> {
		let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
		Ok(manager.open_service(LABEL, access)?)
	}
}
