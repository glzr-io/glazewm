use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};
use windows::Win32::Foundation::HWND;

use crate::user_config::UserConfig;

use super::{EventListener, NativeMonitor, NativeWindow};

pub type WindowHandle = HWND;

pub struct Platform;

impl Platform {
  pub fn monitors() -> Result<Vec<NativeMonitor>> {
    todo!()
  }

  pub fn manageable_windows() -> Result<Vec<NativeWindow>> {
    todo!()
  }

  pub async fn new_event_listener(
    config: Arc<Mutex<UserConfig>>,
    config_changes_rx: UnboundedReceiver<UserConfig>,
  ) -> Result<EventListener> {
    EventListener::start(config, config_changes_rx).await
  }
}
