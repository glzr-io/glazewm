use std::thread::JoinHandle;

use tokio::sync::oneshot;
use tracing::warn;
use tray_icon::{
  menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  Icon, TrayIconBuilder,
};

use crate::common::platform::Platform;

/// Ordinal to IDI_ICON definition from `resources.rc`.
const IDI_ICON: u16 = 0x101;

pub struct SystemTray {
  abort_tx: Option<oneshot::Sender<()>>,
  icon_thread: Option<JoinHandle<anyhow::Result<()>>>,
}

impl SystemTray {
  pub fn new() -> anyhow::Result<Self> {
    let (abort_tx, abort_rx) = oneshot::channel();

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

      Platform::run_message_loop(abort_rx);

      Ok(())
    });

    Ok(Self {
      abort_tx: Some(abort_tx),
      icon_thread: Some(icon_thread),
    })
  }

  /// Destroys the event window and stops the message loop.
  pub fn destroy(&mut self) {
    if let Some(abort_tx) = self.abort_tx.take() {
      if abort_tx.send(()).is_err() {
        warn!("Failed to send abort signal to the event window thread.");
      }
    }

    // Wait for the spawned thread to finish.
    if let Some(window_thread) = self.icon_thread.take() {
      if let Err(err) = window_thread.join() {
        warn!("Failed to join event window thread '{:?}'.", err);
      }
    }
  }
}

impl Drop for SystemTray {
  fn drop(&mut self) {
    self.destroy();
  }
}
