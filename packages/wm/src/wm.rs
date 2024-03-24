use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{
  mpsc::{self, UnboundedReceiver, UnboundedSender},
  Mutex,
};

use crate::{
  common::platform::PlatformEvent,
  user_config::{ParsedConfig, UserConfig},
  wm_command::WmCommand,
  wm_event::WmEvent,
  wm_state::WmState,
};

pub struct WindowManager {
  pub event_rx: UnboundedReceiver<WmEvent>,
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

  pub async fn process_command(&mut self, command: WmCommand) {
    match command {
      WmCommand::CloseWindow => {
        todo!()
      }
      WmCommand::DisableBindingMode(_) => {
        todo!()
      }
      WmCommand::ExitWm => {
        todo!()
      }
      WmCommand::EnableBindingMode(_) => {
        todo!()
      }
      WmCommand::FocusInDirection(_) => {
        todo!()
      }
      WmCommand::FocusRecentWorkspace => {
        todo!()
      }
      WmCommand::FocusWorkspaceInSequence => {
        todo!()
      }
      WmCommand::FocusWorkspace(_) => {
        todo!()
      }
      WmCommand::IgnoreWindow => {
        todo!()
      }
      WmCommand::MoveWindow(_) => {
        todo!()
      }
      WmCommand::MoveWindowToWorkspace(_) => {
        todo!()
      }
      WmCommand::MoveWorkspace(_) => {
        todo!()
      }
      WmCommand::Noop => {
        todo!()
      }
      WmCommand::Redraw => {
        todo!()
      }
      WmCommand::ReloadConfig => {
        todo!()
      }
      WmCommand::ResizeWindowWidth(_) => {
        todo!()
      }
      WmCommand::ResizeWindowHeight(_) => {
        todo!()
      }
      WmCommand::SetTilingDirection(_) => {
        todo!()
      }
      WmCommand::SetWindowBorders(_) => {
        todo!()
      }
      WmCommand::SetWindowFloating => {
        todo!()
      }
      WmCommand::ToggleTilingDirection => {
        todo!()
      }
      WmCommand::ToggleFocusMode => {
        todo!()
      }
    }
  }
}
