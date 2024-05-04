use std::sync::Arc;

use anyhow::{bail, Context, Result};
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
  containers::{commands::redraw, traits::CommonGetters},
  user_config::UserConfig,
  windows::{commands::set_floating, traits::WindowGetters},
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

    if subject_container.is_detached() {
      bail!("Cannot run command because subject container is detached.");
    }

    match command {
      InvokeCommand::AdjustBorders(_) => todo!(),
      InvokeCommand::Close => {
        match subject_container.as_window_container() {
          Ok(window) => window.native().close(),
          _ => Ok(()),
        }
      }
      InvokeCommand::Focus(_) => todo!(),
      InvokeCommand::Ignore => todo!(),
      InvokeCommand::Move(_) => todo!(),
      InvokeCommand::MoveWorkspace { direction } => todo!(),
      InvokeCommand::Resize(_) => todo!(),
      InvokeCommand::SetFloating { centered } => {
        match subject_container.as_window_container() {
          Ok(window) => set_floating(window, &mut state),
          _ => Ok(()),
        }
      }
      InvokeCommand::SetFullscreen => todo!(),
      InvokeCommand::SetMaximized => {
        match subject_container.as_window_container() {
          Ok(window) => window.native().maximize(),
          _ => Ok(()),
        }
      }
      InvokeCommand::SetMinimized => {
        match subject_container.as_window_container() {
          Ok(window) => window.native().minimize(),
          _ => Ok(()),
        }
      }
      InvokeCommand::SetTiling => todo!(),
      InvokeCommand::ShellExec { command } => todo!(),
      InvokeCommand::ToggleFloating { centered } => {
        match subject_container.as_window_container() {
          // TODO: Toggle floating.
          Ok(window) => set_floating(window, &mut state),
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleFullscreen => todo!(),
      InvokeCommand::ToggleMaximized => {
        match subject_container.as_window_container() {
          Ok(window) => {
            if window.native().is_maximized() {
              window.native().restore()
            } else {
              window.native().maximize()
            }
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleMinimized => {
        match subject_container.as_window_container() {
          Ok(window) => {
            if window.native().is_minimized() {
              window.native().restore()
            } else {
              window.native().minimize()
            }
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleTiling => todo!(),
      InvokeCommand::ToggleTilingDirection => todo!(),
      InvokeCommand::WmDisableBindingMode { name } => todo!(),
      InvokeCommand::WmExit => todo!(),
      InvokeCommand::WmEnableBindingMode { name } => todo!(),
      InvokeCommand::WmRedraw => {
        let root_container = state.root_container.clone();
        state.add_container_to_redraw(root_container.into());
        Ok(())
      }
      InvokeCommand::WmReloadConfig => todo!(),
      InvokeCommand::WmToggleFocusMode => todo!(),
    }?;

    redraw(&mut state, config)?;
    sync_native_focus(&mut state)?;

    Ok(())
  }
}
