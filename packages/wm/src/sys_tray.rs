use std::{path::Path, thread::JoinHandle, time::Duration};

use anyhow::Context;
use tokio::sync::mpsc;
use tracing::{info, warn};
use tray_icon::{
  menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  Icon, TrayIconBuilder,
};
use wm_platform::Platform;

/// Ordinal to IDI_ICON definition in embedded resource file.
const IDI_ICON: u16 = 32512;

pub struct SystemTray {
  pub config_reload_rx: mpsc::UnboundedReceiver<()>,
  pub exit_rx: mpsc::UnboundedReceiver<()>,
  icon_thread: Option<JoinHandle<anyhow::Result<()>>>,
}

impl SystemTray {
  pub fn new(config_path: &Path) -> anyhow::Result<Self> {
    let (exit_tx, exit_rx) = mpsc::unbounded_channel();
    let (config_reload_tx, config_reload_rx) = mpsc::unbounded_channel();
    let config_dir = config_path
      .parent()
      .context("Invalid config path.")?
      .to_owned();

    let icon_thread = std::thread::spawn(move || {
      let reload_config_item = MenuItem::new("Reload config", true, None);

      let config_dir_item =
        MenuItem::new("Show config folder", true, None);

      let mut animations_enabled =
        Platform::window_animations_enabled().unwrap_or(true);

      let animations_item = CheckMenuItem::new(
        "Window animations",
        true,
        animations_enabled,
        None,
      );

      let exit_item = MenuItem::new("Exit", true, None);

      let tray_menu = Menu::new();
      tray_menu.append_items(&[
        &reload_config_item,
        &config_dir_item,
        &animations_item,
        &PredefinedMenuItem::separator(),
        &exit_item,
      ])?;

      let icon = Icon::from_resource(IDI_ICON, None)?;
      let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip(format!("GlazeWM v{}", env!("VERSION_NUMBER")))
        .with_icon(icon)
        .build()?;

      let menu_event_rx = MenuEvent::receiver();

      loop {
        if let Ok(event) = menu_event_rx.try_recv() {
          if event.id == reload_config_item.id() {
            config_reload_tx.send(())?;
          } else if event.id == config_dir_item.id() {
            // Open config directory in File Explorer.
            let _ = Platform::open_file_explorer(&config_dir);
          } else if event.id == animations_item.id() {
            // Toggle window animations globally.
            let _ =
              Platform::set_window_animations_enabled(!animations_enabled);

            animations_enabled = !animations_enabled;
          } else if event.id == exit_item.id() {
            exit_tx.send(())?;
          }
        }

        // Run message loop with a delay of 16ms (60fps).
        if Platform::run_message_cycle().is_err() {
          break;
        }

        std::thread::sleep(Duration::from_millis(16));
      }

      Ok(())
    });

    Ok(Self {
      config_reload_rx,
      exit_rx,
      icon_thread: Some(icon_thread),
    })
  }

  /// Destroys the system tray icon and stops its associated message loop.
  pub fn destroy(&mut self) -> anyhow::Result<()> {
    info!("Shutting down system tray.");

    // Wait for the spawned thread to finish.
    if let Some(icon_thread) = self.icon_thread.take() {
      Platform::kill_message_loop(&icon_thread)?;

      icon_thread
        .join()
        .map_err(|_| anyhow::anyhow!("Thread join failed."))??;
    }

    Ok(())
  }
}

impl Drop for SystemTray {
  fn drop(&mut self) {
    if let Err(err) = self.destroy() {
      warn!("Failed to gracefully shut down system tray: {}", err);
    }
  }
}
