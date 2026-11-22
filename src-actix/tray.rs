use color_eyre::Result;
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tokio::sync::oneshot;
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

const APP_NAME: &str = "Obsidian";
const ICON_SIZE: u32 = 32;
const ICON_CENTER: u32 = ICON_SIZE / 2;
const ICON_RADIUS: u32 = 14;
const ICON_COLOR: [u8; 4] = [0xB2, 0x4B, 0xFF, 0xFF];

enum UserEvent {
	Menu(MenuEvent),
	ServerStopped(std::result::Result<(), String>),
}

/// Must run on the main thread; Quit resolves the receiver handed to `serve`.
pub fn run<F>(panel_url: &str, serve: F) -> Result<()>
where
	F: FnOnce(oneshot::Receiver<()>) -> Result<()> + Send + 'static,
{
	if !display_available() {
		tracing::warn!("no display detected, running without a system tray");
		let (_shutdown, shutdown_rx) = oneshot::channel();
		return serve(shutdown_rx);
	}

	#[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
	let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
	#[cfg(target_os = "macos")]
	{
		use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
		event_loop.set_activation_policy(ActivationPolicy::Accessory);
	}
	let proxy = event_loop.create_proxy();
	let (shutdown_tx, shutdown_rx) = oneshot::channel();
	let server_proxy = proxy.clone();
	std::thread::Builder::new()
		.name("panel-server".into())
		.spawn(move || {
			let outcome = serve(shutdown_rx).map_err(|error| format!("{error:#}"));
			let _ = server_proxy.send_event(UserEvent::ServerStopped(outcome));
		})?;
	MenuEvent::set_event_handler(Some(move |event| {
		let _ = proxy.send_event(UserEvent::Menu(event));
	}));

	let panel_url = panel_url.to_owned();
	let mut shutdown_tx = Some(shutdown_tx);
	let mut tray: Option<TrayIcon> = None;
	let mut menu_ids: Option<(MenuId, MenuId)> = None;
	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Wait;
		match event {
			Event::NewEvents(StartCause::Init) => match build_tray(&panel_url) {
				Ok((icon, open, quit)) => {
					tray = Some(icon);
					menu_ids = Some((open, quit));
				}
				Err(error) => tracing::error!(%error, "failed to create the system tray icon"),
			},
			Event::UserEvent(UserEvent::Menu(event)) => {
				let Some((open, quit)) = &menu_ids else {
					return;
				};
				if event.id == *open {
					if let Err(error) = open::that(&panel_url) {
						tracing::error!(%error, "failed to open the panel in a browser");
					}
				} else if event.id == *quit {
					tray.take();
					if let Some(tx) = shutdown_tx.take() {
						let _ = tx.send(());
					}
				}
			}
			Event::UserEvent(UserEvent::ServerStopped(outcome)) => {
				tray.take();
				if let Err(error) = outcome {
					tracing::error!(%error, "panel server stopped with an error");
					std::process::exit(1);
				}
				*control_flow = ControlFlow::Exit;
			}
			_ => {}
		}
	})
}

fn display_available() -> bool {
	if cfg!(target_os = "linux") {
		std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some()
	} else {
		true
	}
}

fn build_tray(panel_url: &str) -> Result<(TrayIcon, MenuId, MenuId)> {
	let status = MenuItem::new(format!("Running at {panel_url}"), false, None);
	let open = MenuItem::new("Open Panel", true, None);
	let quit = MenuItem::new("Quit", true, None);
	let menu = Menu::new();
	menu.append(&status)?;
	menu.append(&PredefinedMenuItem::separator())?;
	menu.append(&open)?;
	menu.append(&quit)?;
	let tray = TrayIconBuilder::new()
		.with_menu(Box::new(menu))
		.with_tooltip(APP_NAME)
		.with_icon(icon()?)
		.build()?;
	Ok((tray, open.id().clone(), quit.id().clone()))
}

fn icon() -> Result<Icon> {
	let pixels = (0..ICON_SIZE * ICON_SIZE)
		.flat_map(|index| {
			let (x, y) = (index % ICON_SIZE, index / ICON_SIZE);
			let inside = x.abs_diff(ICON_CENTER) + y.abs_diff(ICON_CENTER) <= ICON_RADIUS;
			if inside { ICON_COLOR } else { [0; 4] }
		})
		.collect();
	Ok(Icon::from_rgba(pixels, ICON_SIZE, ICON_SIZE)?)
}
