use std::sync::Arc;

use tokio::sync::Mutex;
use windows::Win32::UI::WindowsAndMessaging::{
  GetDesktopWindow, GetForegroundWindow,
};

use crate::user_config::UserConfig;

use super::{
  native_monitor, native_window, EventListener, NativeMonitor,
  NativeWindow, SingleInstance,
};

pub struct Platform;

impl Platform {
  pub fn foreground_window() -> NativeWindow {
    let handle = unsafe { GetForegroundWindow() };
    NativeWindow::new(handle)
  }

  pub fn desktop_window() -> NativeWindow {
    let handle = unsafe { GetDesktopWindow() };
    NativeWindow::new(handle)
  }

  pub fn monitors() -> anyhow::Result<Vec<NativeMonitor>> {
    native_monitor::available_monitors()
  }

  pub fn nearest_monitor(
    window: &NativeWindow,
  ) -> anyhow::Result<NativeMonitor> {
    native_monitor::nearest_monitor(window.handle)
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
    config: &Arc<Mutex<UserConfig>>,
  ) -> anyhow::Result<EventListener> {
    EventListener::start(config).await
  }

  pub fn new_single_instance() -> anyhow::Result<SingleInstance> {
    SingleInstance::new()
  }
}
