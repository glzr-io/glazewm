use std::thread::JoinHandle;

use tracing::{error, info};
use tray_icon::{
  menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  Icon, TrayIconBuilder,
};

use crate::common::platform::Platform;

/// Ordinal to IDI_ICON definition from `resources.rc`.
const IDI_ICON: u16 = 0x101;

pub struct SystemTray {
  icon_thread: Option<JoinHandle<anyhow::Result<()>>>,
}

impl SystemTray {
  pub fn new() -> anyhow::Result<Self> {
    let icon_thread = std::thread::spawn(move || {
      let tray_menu = Menu::new();

      let quit_item = MenuItem::new("Quit", true, None);

      tray_menu.append_items(&[
        &PredefinedMenuItem::about(
          None,
          Some(AboutMetadata {
            name: Some("test".to_string()),
            copyright: Some("Copyright test".to_string()),
            ..Default::default()
          }),
        ),
        &PredefinedMenuItem::separator(),
        &quit_item,
      ])?;

      let menu_event_rx = MenuEvent::receiver();

      let icon = Icon::from_resource(IDI_ICON, None)?;
      let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("test")
        .with_icon(icon)
        .build()?;

      Platform::run_message_loop();

      Ok(())
    });

    Ok(Self {
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
      error!("Failed to gracefully shut down system tray: {}", err);
    }
  }
}
