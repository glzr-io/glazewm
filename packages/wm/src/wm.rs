use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{
  mpsc::{self, UnboundedReceiver, UnboundedSender},
  Mutex,
};

use crate::{
  common::platform::PlatformEvent, user_config::UserConfig,
  wm_command::WmCommand, wm_event::WmEvent, wm_state::WmState,
};

pub struct WindowManager {
  pub event_rx: UnboundedReceiver<WmEvent>,
  pub state: Arc<Mutex<WmState>>,
}

impl WindowManager {
  pub fn start(
    config: Arc<Mutex<UserConfig>>,
    config_changes_tx: UnboundedSender<UserConfig>,
  ) -> Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let mut state = WmState::new(config, config_changes_tx, event_tx);
    state.populate()?;

    Ok(Self {
      event_rx,
      state: Arc::new(Mutex::new(state)),
    })
  }

  // pub fn init() -> Result<()> {
  //   let foreground_window = Platform::foreground_window();

  //   let monitors = Platform::monitors()?;
  //   let manageable_windows = Platform::manageable_windows()?;

  //   Ok(())
  // }

  pub async fn process_event(&mut self, event: PlatformEvent) {
    todo!()
  }

  pub async fn process_command(&mut self, command: WmCommand) {
    todo!()
  }
}
