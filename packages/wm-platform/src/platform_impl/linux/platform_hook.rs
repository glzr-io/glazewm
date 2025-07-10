use smithay::utils::SERIAL_COUNTER;
use wm_common::{
  BindingModeConfig, InvokeCommand, KeybindingConfig, ParsedConfig,
};

use super::{
  display::{DisplayHook, EventThreadDisplayHook},
  event_loop::EventLoop,
  keyboard::{EventThreadKeyboardHook, KeyboardHook},
  mouse::{EventThreadMouseHook, MouseHook},
  window::{EventThreadWindowEventHook, WindowEventHook},
  NativeWindow,
};
use crate::WindowEventType;

pub struct PlatformHook {
  event_loop: EventLoop,
}

impl PlatformHook {
  pub fn dedicated(config: &ParsedConfig) -> anyhow::Result<Self> {
    let event_loop = EventLoop::new(config);

    Ok(Self { event_loop })
  }

  // Windows implementation is async
  #[allow(clippy::unused_async)]
  pub async fn create_mouse_listener(&self) -> anyhow::Result<MouseHook> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    self.event_loop.dispatch(move |data| {
      let hook = EventThreadMouseHook::new(tx);
      if let Err(e) = data.state.hooks.register_mouse_hook(hook) {
        tracing::error!("Failed to register mouse hook: {}", e);
      }
    })?;

    Ok(MouseHook::new(rx))
  }

  #[allow(clippy::unused_async)]
  pub async fn create_display_listener(
    &self,
  ) -> anyhow::Result<DisplayHook> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    self.event_loop.dispatch(move |data| {
      let hook = EventThreadDisplayHook::new(tx);
      if let Err(e) = data.state.hooks.register_display_hook(hook) {
        tracing::error!("Failed to register display hook: {}", e);
      }
    })?;
    Ok(DisplayHook::new(rx))
  }

  #[allow(clippy::unused_async)]
  pub async fn with_window_events(
    &self,
    events: WindowEventType,
  ) -> anyhow::Result<WindowEventHook> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    self.event_loop.dispatch(move |data| {
      let hook = EventThreadWindowEventHook::new(tx, events);
      if let Err(e) = data.state.hooks.register_window_hook(hook) {
        tracing::error!("Failed to register window hook: {}", e);
      }
    })?;
    Ok(WindowEventHook::new(rx))
  }

  #[allow(clippy::unused_async)]
  pub async fn create_keyboard_listener(
    &self,
    keybindings: &[KeybindingConfig],
  ) -> anyhow::Result<KeyboardHook> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let keybindings = keybindings.to_vec(); // Clone to move into the closure
    self.event_loop.dispatch(move |data| {
      let hook = EventThreadKeyboardHook::new(tx, keybindings);
      if let Err(e) = data.state.hooks.register_keyboard_hook(hook) {
        tracing::error!("Failed to register keyboard hook: {}", e);
      }
    })?;
    Ok(KeyboardHook::new(rx))
  }

  pub fn update_keybinds(
    &self,
    keybindings: &[KeybindingConfig],
    binding_modes: &[BindingModeConfig],
    paused: bool,
  ) -> anyhow::Result<()> {
    let keybindings = if paused {
      &keybindings
        .iter()
        .filter(|config| {
          config.commands.contains(&InvokeCommand::WmTogglePause)
        })
        .cloned()
        .collect::<Vec<_>>()
    } else {
      match binding_modes.first() {
        Some(binding_mode) => &binding_mode.keybindings,
        None => keybindings,
      }
    };

    let keybindings = keybindings.to_vec();
    self.event_loop.dispatch(move |data| {
      if let Err(e) = data.state.hooks.update_keybinds(keybindings) {
        tracing::error!("Failed to update keybinds: {}", e);
      }
    })?;
    Ok(())
  }

  pub fn update_mouse(&self, enable: bool) -> anyhow::Result<()> {
    self.event_loop.dispatch(move |data| {
      if let Err(e) = data.state.hooks.update_mouse(enable) {
        tracing::error!("Failed to update mouse: {}", e);
      }
    })?;
    Ok(())
  }

  pub fn show_error_dialog(&self, title: &str, message: &str) {
    todo!()
  }

  #[must_use]
  pub fn desktop_window(&self) -> NativeWindow {
    todo!()
  }

  #[must_use]
  pub fn is_foreground_window(&self, _: &NativeWindow) -> bool {
    false
  }

  pub fn mouse_position(&self) -> anyhow::Result<wm_common::Point> {
    todo!()
  }

  pub fn set_cursor_pos(&self, x: i32, y: i32) -> anyhow::Result<()> {
    self.event_loop.dispatch(move |data| {
      if let Some(pointer) = data.state.seat.get_pointer() {
        let point = smithay::utils::Point::new(f64::from(x), f64::from(y));
        let surface = data.state.surface_under(point);
        #[allow(clippy::cast_possible_truncation)]
        let event = smithay::input::pointer::MotionEvent {
          location: point,
          serial: SERIAL_COUNTER.next_serial(),
          time: data.state.clock.now().as_millis(),
        };
        pointer.motion(&mut data.state, surface, &event);
      }
    })?;
    Ok(())
  }
}
