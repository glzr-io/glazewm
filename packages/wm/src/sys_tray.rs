use std::{
  fmt::{self, Display},
  path::Path,
  str::FromStr,
};

use anyhow::Context;
use tokio::sync::mpsc;
use tray_icon::{
  menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  Icon, TrayIcon, TrayIconBuilder,
};
use wm_platform::{Dispatcher, ThreadBound};

#[derive(Debug, Clone, Eq, PartialEq)]
enum TrayMenuId {
  ReloadConfig,
  ShowConfigFolder,
  Exit,
}

impl Display for TrayMenuId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      TrayMenuId::ReloadConfig => write!(f, "reload_config"),
      TrayMenuId::ShowConfigFolder => write!(f, "show_config_folder"),
      TrayMenuId::Exit => write!(f, "exit"),
    }
  }
}

impl FromStr for TrayMenuId {
  type Err = anyhow::Error;

  fn from_str(event: &str) -> Result<Self, Self::Err> {
    match event {
      "show_config_folder" => Ok(Self::ShowConfigFolder),
      "reload_config" => Ok(Self::ReloadConfig),
      "exit" => Ok(Self::Exit),
      _ => anyhow::bail!("Invalid tray menu event: {}", event),
    }
  }
}

pub struct SystemTray {
  pub config_reload_rx: mpsc::UnboundedReceiver<()>,
  pub exit_rx: mpsc::UnboundedReceiver<()>,
  icon_thread: Option<std::thread::JoinHandle<()>>,
  tray_icon: ThreadBound<TrayIcon>,
}

impl SystemTray {
  /// Install the system tray on the main thread after the run loop starts.
  pub fn new(
    config_path: &Path,
    dispatcher: &Dispatcher,
  ) -> anyhow::Result<Self> {
    let (exit_tx, exit_rx) = mpsc::unbounded_channel();
    let (config_reload_tx, config_reload_rx) = mpsc::unbounded_channel();
    let _config_dir = config_path
      .parent()
      .context("Invalid config path.")?
      .to_owned();

    let tray_icon = dispatcher.dispatch_sync(move || {
      let tray_icon = Self::create_tray_icon().unwrap();
      ThreadBound::new(tray_icon, dispatcher.clone())
    })?;

    // Spawn thread to handle menu events and forward them to channels
    let icon_thread = std::thread::spawn(move || {
      let menu_event_rx = MenuEvent::receiver();

      while let Ok(event) = menu_event_rx.recv() {
        let event_res = match TrayMenuId::from_str(event.id.as_ref()) {
          Ok(TrayMenuId::ShowConfigFolder) => {
            // TODO: Implement show config folder
            tracing::info!("Show config folder requested");
            Ok(())
          }
          Ok(TrayMenuId::ReloadConfig) => config_reload_tx.send(()),
          Ok(TrayMenuId::Exit) => exit_tx.send(()),
          Err(err) => {
            tracing::warn!("Failed to parse tray menu event: {}", err);
            continue;
          }
        };

        if let Err(err) = event_res {
          tracing::warn!("Failed to send tray menu event: {}", err);
        }
      }
    });

    Ok(Self {
      config_reload_rx,
      exit_rx,
      icon_thread: Some(icon_thread),
      tray_icon,
    })
  }

  fn create_tray_icon() -> anyhow::Result<TrayIcon> {
    let reload_config_item = MenuItem::with_id(
      TrayMenuId::ReloadConfig,
      "Reload config",
      true,
      None,
    );

    let config_dir_item = MenuItem::with_id(
      TrayMenuId::ShowConfigFolder,
      "Show config folder",
      true,
      None,
    );

    let exit_item =
      MenuItem::with_id(TrayMenuId::Exit, "Exit", true, None);

    let tray_menu = Menu::new();
    tray_menu.append_items(&[
      &reload_config_item,
      &config_dir_item,
      &PredefinedMenuItem::separator(),
      &exit_item,
    ])?;

    let path = concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/../../resources/assets/icon.png"
    );

    let icon = Self::load_icon(Path::new(path))?;

    let tray_icon = TrayIconBuilder::new()
      .with_menu(Box::new(tray_menu))
      .with_tooltip(format!("GlazeWM v{}", env!("VERSION_NUMBER")))
      .with_icon(icon)
      .build()?;

    Ok(tray_icon)
  }

  fn load_icon(path: &Path) -> anyhow::Result<Icon> {
    let (icon_rgba, icon_width, icon_height) = {
      let image = image::open(path)
        .context("Failed to open icon path.")?
        .into_rgba8();

      let (width, height) = image.dimensions();
      let rgba = image.into_raw();
      (rgba, width, height)
    };

    Ok(tray_icon::Icon::from_rgba(
      icon_rgba,
      icon_width,
      icon_height,
    )?)
  }

  /// Destroys the system tray icon and stops its associated message loop.
  pub fn destroy(&mut self) -> anyhow::Result<()> {
    tracing::info!("Shutting down system tray.");
    // Tray icon and event thread will be cleaned up when the app exits
    Ok(())
  }
}

impl Drop for SystemTray {
  fn drop(&mut self) {
    if let Err(err) = self.destroy() {
      tracing::warn!(
        "Failed to gracefully shut down system tray: {}",
        err
      );
    }
  }
}
