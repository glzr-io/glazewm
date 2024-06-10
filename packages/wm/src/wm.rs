use anyhow::Context;
use tokio::sync::mpsc::{self};
use uuid::Uuid;

use crate::common::commands::platform_sync;
use crate::common::events::handle_mouse_move;
use crate::{
  app_command::InvokeCommand,
  common::{
    events::{
      handle_display_settings_changed, handle_window_destroyed,
      handle_window_focused, handle_window_hidden,
      handle_window_location_changed, handle_window_minimize_ended,
      handle_window_minimized, handle_window_moved_or_resized,
      handle_window_shown,
    },
    platform::PlatformEvent,
  },
  containers::traits::CommonGetters,
  user_config::UserConfig,
  wm_event::WmEvent,
  wm_state::WmState,
};

pub struct WindowManager {
  pub event_rx: mpsc::UnboundedReceiver<WmEvent>,
  pub state: WmState,
}

impl WindowManager {
  pub fn new(config: &UserConfig) -> anyhow::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let mut state = WmState::new(event_tx);
    state.populate(&config)?;

    Ok(Self { event_rx, state })
  }

  pub fn process_event(
    &mut self,
    event: PlatformEvent,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    let state = &mut self.state;

    match event {
      PlatformEvent::DisplaySettingsChanged => {
        handle_display_settings_changed(state, config)
      }
      PlatformEvent::KeybindingTriggered(kb_config) => {
        // Return early since we don't want to redraw twice.
        self.process_commands(kb_config.commands, None, config)?;
        return Ok(());
      }
      PlatformEvent::MouseMove(event) => {
        handle_mouse_move(event, state, config)
      }
      PlatformEvent::WindowDestroyed(window) => {
        handle_window_destroyed(window, state)
      }
      PlatformEvent::WindowFocused(window) => {
        handle_window_focused(window, state, config)
      }
      PlatformEvent::WindowHidden(window) => {
        handle_window_hidden(window, state)
      }
      PlatformEvent::WindowLocationChanged(window) => {
        handle_window_location_changed(window, state, config)
      }
      PlatformEvent::WindowMinimized(window) => {
        handle_window_minimized(window, state, config)
      }
      PlatformEvent::WindowMinimizeEnded(window) => {
        handle_window_minimize_ended(window, state, config)
      }
      PlatformEvent::WindowMovedOrResized(window) => {
        handle_window_moved_or_resized(window, state)
      }
      PlatformEvent::WindowShown(window) => {
        handle_window_shown(window, state, config)
      }
      PlatformEvent::WindowTitleChanged(_) => Ok(()),
    }?;

    platform_sync(state, config)?;

    Ok(())
  }

  pub fn process_commands(
    &mut self,
    commands: Vec<InvokeCommand>,
    subject_container_id: Option<Uuid>,
    config: &mut UserConfig,
  ) -> anyhow::Result<Uuid> {
    let state = &mut self.state;

    // Get the container to run WM commands with.
    let mut subject_container = match subject_container_id {
      Some(id) => state.container_by_id(id).with_context(|| {
        format!("No container found with the given ID '{}'.", id)
      })?,
      None => state
        .focused_container()
        .context("No subject container for command.")?,
    };

    for command in commands {
      command.run(subject_container.clone(), state, config)?;

      // Update the subject container in case the container type changes.
      // For example, when going from a tiling to a floating window.
      subject_container = match subject_container.is_detached() {
        false => subject_container,
        true => match state.container_by_id(subject_container.id()) {
          Some(container) => container,
          None => break,
        },
      }
    }

    platform_sync(state, config)?;

    Ok(subject_container.id())
  }
}
