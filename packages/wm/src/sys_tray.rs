use std::{thread::JoinHandle, time::Duration};

use tracing::{error, info};
use tray_icon::{
  menu::{
    AboutMetadata, CheckMenuItem, Menu, MenuEvent, MenuItem,
    PredefinedMenuItem,
  },
  Icon, TrayIconBuilder,
};

use crate::common::platform::Platform;

/// Ordinal to IDI_ICON definition from `resources.rc` file.
const IDI_ICON: u16 = 0x101;

pub struct SystemTray {
  icon_thread: Option<JoinHandle<anyhow::Result<()>>>,
}

impl SystemTray {
  pub fn new() -> anyhow::Result<Self> {
    let icon_thread = std::thread::spawn(move || {
      let tray_menu = Menu::new();

      let quit_item = MenuItem::new("Quit", true, None);
      let animations_item =
        CheckMenuItem::new("Window animations", true, false, None);

      tray_menu.append_items(&[
        &animations_item,
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

      let icon = Icon::from_resource(IDI_ICON, None)?;
      let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip(format!("GlazeWM v{}", env!("CARGO_PKG_VERSION")))
        .with_icon(icon)
        .build()?;

      let menu_event_rx = MenuEvent::receiver();

      loop {
        if let Ok(event) = menu_event_rx.try_recv() {
          if event.id == quit_item.id() {
            quit_item.set_text("New title");
          } else if event.id == animations_item.id() {
            animations_item.set_checked(animations_item.is_checked());
          }
        }

        // Add delay of 16ms (60fps) to reduce cpu load.
        Platform::run_message_cycle();
        std::thread::sleep(Duration::from_millis(16));
      }
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
