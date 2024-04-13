use std::{ops::DerefMut, sync::Arc};

use anyhow::Result;
use tokio::sync::{
  mpsc::{self},
  Mutex,
};
use tracing::error;

use crate::{
  app_command::InvokeCommand,
  common::{
    events::{handle_window_destroyed, handle_window_shown},
    platform::PlatformEvent,
  },
  containers::commands::redraw,
  user_config::UserConfig,
  wm_event::WmEvent,
  wm_state::WmState,
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

  pub async fn process_event(
    &mut self,
    event: PlatformEvent,
    config: &mut UserConfig,
  ) {
    let mut state = self.state.lock().await;

    let res = match event {
      PlatformEvent::DisplaySettingsChanged => Ok(()),
      PlatformEvent::KeybindingTriggered(_) => Ok(()),
      PlatformEvent::MouseMove => Ok(()),
      PlatformEvent::WindowDestroyed(w) => {
        handle_window_destroyed(w, state.deref_mut(), config)
      }
      PlatformEvent::WindowFocused(_) => Ok(()),
      PlatformEvent::WindowHidden(_) => Ok(()),
      PlatformEvent::WindowLocationChanged(_) => Ok(()),
      PlatformEvent::WindowMinimized(_) => Ok(()),
      PlatformEvent::WindowMinimizeEnded(_) => Ok(()),
      PlatformEvent::WindowMovedOrResized(_) => Ok(()),
      PlatformEvent::WindowShown(w) => {
        handle_window_shown(w, state.deref_mut(), config)
      }
      PlatformEvent::WindowTitleChanged(_) => Ok(()),
    };

    if let Err(err) = res {
      error!("Failed to process event: {:?}", err);
    }
  }

  pub async fn process_command(
    &mut self,
    command: InvokeCommand,
    config: &mut UserConfig,
  ) {
    let mut state = self.state.lock().await;

    let res = match command {
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
    };

    if let Err(err) = res {
      error!("Failed to process command: {:?}", err);
    }
  }
}
