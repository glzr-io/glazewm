use std::{
  ops::{Deref, DerefMut},
  sync::Arc,
};

use anyhow::Result;
use tokio::sync::{
  mpsc::{self},
  Mutex,
};

use crate::{
  app_command::InvokeCommand, common::platform::PlatformEvent,
  containers::commands::redraw, user_config::UserConfig,
  wm_event::WmEvent, wm_state::WmState,
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

  pub async fn process_command(
    &mut self,
    command: InvokeCommand,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    let mut state = self.state.lock().await;

    match command {
      InvokeCommand::AdjustBorders(_) => todo!(),
      InvokeCommand::Close => todo!(),
      InvokeCommand::Focus(_) => todo!(),
      InvokeCommand::Ignore => todo!(),
      InvokeCommand::Move(_) => todo!(),
      InvokeCommand::MoveWorkspace { direction } => todo!(),
      InvokeCommand::Resize(_) => todo!(),
      InvokeCommand::SetFloating { centered } => todo!(),
      InvokeCommand::SetFullscreen => todo!(),
      InvokeCommand::SetMaximized => todo!(),
      InvokeCommand::SetMinimized => todo!(),
      InvokeCommand::SetTiling => todo!(),
      InvokeCommand::ShellExec { command } => todo!(),
      InvokeCommand::ToggleFloating { centered } => todo!(),
      InvokeCommand::ToggleFullscreen => todo!(),
      InvokeCommand::ToggleMaximized => todo!(),
      InvokeCommand::ToggleMinimized => todo!(),
      InvokeCommand::ToggleTiling => todo!(),
      InvokeCommand::ToggleTilingDirection => todo!(),
      InvokeCommand::WmDisableBindingMode { name } => todo!(),
      InvokeCommand::WmExit => todo!(),
      InvokeCommand::WmEnableBindingMode { name } => todo!(),
      InvokeCommand::WmRedraw => redraw(state.deref_mut(), &config),
      InvokeCommand::WmReloadConfig => todo!(),
      InvokeCommand::WmToggleFocusMode => todo!(),
    }
  }
}
