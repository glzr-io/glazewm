use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{
  mpsc::{self},
  Mutex,
};

use crate::{
  app_command::InvokeCommand, common::platform::PlatformEvent,
  user_config::UserConfig, wm_event::WmEvent, wm_state::WmState,
};

pub struct WindowManager {
  pub event_rx: mpsc::UnboundedReceiver<WmEvent>,
  pub state: Arc<Mutex<WmState>>,
}

impl WindowManager {
  pub async fn start(config: &Arc<Mutex<UserConfig>>) -> Result<Self> {
    let config = config.lock().await;
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let mut state = WmState::new(event_tx);
    state.populate(&config)?;

    Ok(Self {
      event_rx,
      state: Arc::new(Mutex::new(state)),
    })
  }

  pub async fn process_event(&mut self, event: PlatformEvent) {
    // TODO
  }

  pub async fn process_command(&mut self, command: InvokeCommand) {
    // TODO
  }
}
