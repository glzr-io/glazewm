use tokio::sync::mpsc::{self, UnboundedReceiver};
use wm_common::{
  BindingModeConfig, InvokeCommand, KeybindingConfig, ParsedConfig, Point,
};

use super::{EventWindow, NativeWindow};

#[derive(Debug)]
pub enum PlatformEvent {
  DisplaySettingsChanged,
  KeybindingTriggered(KeybindingConfig),
  MouseMove(MouseMoveEvent),
  WindowDestroyed(NativeWindow),
  WindowFocused(NativeWindow),
  WindowHidden(NativeWindow),
  WindowLocationChanged(NativeWindow),
  WindowMinimized(NativeWindow),
  WindowMinimizeEnded(NativeWindow),
  WindowMovedOrResizedEnd(NativeWindow),
  WindowMovedOrResizedStart(NativeWindow),
  WindowShown(NativeWindow),
  WindowTitleChanged(NativeWindow),
}

#[derive(Debug, Clone)]
pub struct MouseMoveEvent {
  /// Location of mouse with 0,0 being the top-left corner of the primary
  /// monitor.
  pub point: Point,

  /// Whether either left or right-click is currently pressed.
  pub is_mouse_down: bool,
}

pub struct EventListener {
  pub event_rx: UnboundedReceiver<PlatformEvent>,
  event_window: EventWindow,
}

impl EventListener {
  /// Initializes listener for platform events.
  ///
  /// Returns an instance of `EventListener`.
  pub fn start(config: &ParsedConfig) -> anyhow::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let event_window = EventWindow::new(
      &event_tx,
      &config.keybindings,
      config.general.focus_follows_cursor,
    )?;

    Ok(Self {
      event_rx,
      event_window,
    })
  }

  /// Updates the event listener with the latest user config and the
  /// currently active binding modes.
  pub fn update(
    &mut self,
    config: &ParsedConfig,
    binding_modes: &[BindingModeConfig],
    paused: bool,
  ) {
    // Modify keybindings based on active binding modes and paused state.
    let keybindings = if paused {
      &config
        .keybindings
        .iter()
        .filter(|config| {
          config.commands.contains(&InvokeCommand::WmTogglePause)
        })
        .cloned()
        .collect::<Vec<_>>()
    } else {
      match binding_modes.first() {
        Some(binding_mode) => &binding_mode.keybindings,
        None => &config.keybindings,
      }
    };

    self
      .event_window
      .update(keybindings, config.general.focus_follows_cursor && !paused);
  }
}
