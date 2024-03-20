use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc::UnboundedReceiver, Mutex};

use crate::user_config::UserConfig;

use super::{native_monitor, EventListener, NativeMonitor, NativeWindow};

pub struct Platform;

impl Platform {
  pub fn monitors() -> Vec<NativeMonitor> {
    native_monitor::available_monitors()
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
