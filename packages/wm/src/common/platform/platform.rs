use std::sync::Arc;

use tokio::sync::{mpsc::UnboundedReceiver, Mutex};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

use crate::user_config::UserConfig;

use super::{
  native_monitor, native_window, EventListener, NativeMonitor,
  NativeWindow,
};

pub struct Platform;

impl Platform {
  pub fn foreground_window() -> NativeWindow {
    let handle = unsafe { GetForegroundWindow() };
    NativeWindow::new(handle)
  }

  pub fn monitors() -> anyhow::Result<Vec<NativeMonitor>> {
    native_monitor::available_monitors()
  }

  pub fn nearest_monitor(window: NativeWindow) -> Option<NativeMonitor> {
    native_monitor::available_monitors()
      .ok()
      .and_then(|m| m.first().cloned())
  }

  pub fn manageable_windows() -> anyhow::Result<Vec<NativeWindow>> {
    Ok(
      native_window::available_windows()?
        .into_iter()
        .filter(|w| w.is_manageable())
        .collect(),
    )
  }

  pub async fn new_event_listener(
    config: Arc<Mutex<UserConfig>>,
    config_changes_rx: UnboundedReceiver<UserConfig>,
  ) -> anyhow::Result<EventListener> {
    EventListener::start(config, config_changes_rx).await
  }
}
