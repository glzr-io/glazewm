use std::{ops::DerefMut, sync::Arc};

use anyhow::{Context, Result};
use tokio::sync::{
  mpsc::{self},
  Mutex,
};

use crate::{
  app_command::InvokeCommand,
  common::{
    events::{
      handle_window_destroyed, handle_window_focused,
      handle_window_hidden, handle_window_shown,
    },
    platform::PlatformEvent,
  },
  containers::{commands::redraw, traits::CommonGetters},
  user_config::UserConfig,
  windows::traits::WindowGetters,
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
  ) -> anyhow::Result<()> {
    match event {
      PlatformEvent::DisplaySettingsChanged => Ok(()),
      PlatformEvent::KeybindingTriggered(kb_config) => {
        for command in kb_config.commands {
          self.process_command(command, config).await?;
        }
        Ok(())
      }
      PlatformEvent::MouseMove(_) => Ok(()),
      PlatformEvent::WindowDestroyed(window) => handle_window_destroyed(
        window,
        self.state.lock().await.deref_mut(),
        config,
      ),
      PlatformEvent::WindowFocused(window) => handle_window_focused(
        window,
        self.state.lock().await.deref_mut(),
        config,
      ),
      PlatformEvent::WindowHidden(window) => handle_window_hidden(
        window,
        self.state.lock().await.deref_mut(),
        config,
      ),
      PlatformEvent::WindowLocationChanged(_) => Ok(()),
      PlatformEvent::WindowMinimized(_) => Ok(()),
      PlatformEvent::WindowMinimizeEnded(_) => Ok(()),
      PlatformEvent::WindowMovedOrResized(_) => Ok(()),
      PlatformEvent::WindowShown(window) => handle_window_shown(
        window,
        self.state.lock().await.deref_mut(),
        config,
      ),
      PlatformEvent::WindowTitleChanged(_) => Ok(()),
    }
  }

  pub async fn process_command(
    &mut self,
    command: InvokeCommand,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    let mut state = self.state.lock().await;

    let subject_container = state
      .focused_container()
      .context("No subject container for command.")?;

    match command {
      InvokeCommand::AdjustBorders(_) => todo!(),
      InvokeCommand::Close => {
        if let Ok(window) = subject_container.as_window_container() {
          window.native().close()?
        }
        Ok(())
      }
      InvokeCommand::Focus(_) => todo!(),
      InvokeCommand::Ignore => todo!(),
      InvokeCommand::Move(_) => todo!(),
      InvokeCommand::MoveWorkspace { direction } => todo!(),
      InvokeCommand::Resize(_) => todo!(),
      InvokeCommand::SetFloating { centered } => todo!(),
      InvokeCommand::SetFullscreen => todo!(),
      InvokeCommand::SetMaximized => {
        if let Ok(window) = subject_container.as_window_container() {
          window.native().maximize()?
        }
        Ok(())
      }
      InvokeCommand::SetMinimized => {
        if let Ok(window) = subject_container.as_window_container() {
          window.native().minimize()?
        }
        Ok(())
      }
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
