use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::{
  mpsc::{self},
  Mutex,
};
use uuid::Uuid;

use crate::{
  app_command::InvokeCommand,
  common::{
    commands::sync_native_focus,
    events::{
      handle_window_destroyed, handle_window_focused,
      handle_window_hidden, handle_window_shown,
    },
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
  ) -> anyhow::Result<()> {
    let mut state = self.state.lock().await;

    match event {
      PlatformEvent::DisplaySettingsChanged => Ok(()),
      PlatformEvent::KeybindingTriggered(kb_config) => {
        drop(state);
        for command in kb_config.commands {
          // TODO: Postpone redraw + focus sync till after all commands are run.
          self.process_command(command, None, config).await?;
        }
        return Ok(());
      }
      PlatformEvent::MouseMove(_) => Ok(()),
      PlatformEvent::WindowDestroyed(window) => {
        handle_window_destroyed(window, &mut state)
      }
      PlatformEvent::WindowFocused(window) => {
        handle_window_focused(window, &mut state, config)
      }
      PlatformEvent::WindowHidden(window) => {
        handle_window_hidden(window, &mut state, config)
      }
      PlatformEvent::WindowLocationChanged(_) => Ok(()),
      PlatformEvent::WindowMinimized(_) => Ok(()),
      PlatformEvent::WindowMinimizeEnded(_) => Ok(()),
      PlatformEvent::WindowMovedOrResized(_) => Ok(()),
      PlatformEvent::WindowShown(window) => {
        handle_window_shown(window, &mut state, config)
      }
      PlatformEvent::WindowTitleChanged(_) => Ok(()),
    }?;

    redraw(&mut state, config)?;
    sync_native_focus(&mut state)?;

    Ok(())
  }

  pub async fn process_command(
    &mut self,
    command: InvokeCommand,
    subject_container_id: Option<Uuid>,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    let mut state = self.state.lock().await;

    let subject_container = subject_container_id
      .map(|id| state.container_by_id(id))
      .context("No container found with the given ID.")?
      .or_else(|| state.focused_container())
      .context("No subject container for command.")?;

    command.run(subject_container, &mut state, config)?;

    redraw(&mut state, config)?;
    sync_native_focus(&mut state)?;

    Ok(())
  }
}
