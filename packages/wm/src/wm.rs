use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{
  mpsc::{self, UnboundedReceiver, UnboundedSender},
  Mutex,
};
use wineventhook::WindowEvent;

use crate::{
  user_config::UserConfig, wm_event::WmEvent, wm_state::WmState,
};

pub struct WindowManager {
  pub event_rx: UnboundedReceiver<WmEvent>,
  pub state: Arc<Mutex<WmState>>,
}

impl WindowManager {
  pub async fn start(
    config: Arc<Mutex<UserConfig>>,
    config_changes_tx: UnboundedSender<UserConfig>,
  ) -> Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    Ok(Self {
      event_rx,
      state: Arc::new(Mutex::new(WmState::new(config))),
    })
  }

  pub async fn process_event(&mut self, event: WindowEvent) {
    todo!()
  }
}
