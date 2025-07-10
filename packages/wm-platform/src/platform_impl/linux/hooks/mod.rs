use tokio::sync::mpsc::error::SendError;

use crate::WindowEvent;

mod display;
mod keyboard;
mod mouse;
mod window;

pub trait Hook {
  type Event;

  fn dispatch(
    &self,
    event: Self::Event,
  ) -> Result<(), SendError<Self::Event>>;
}

#[derive(Default, Debug)]
pub struct Hooks {
  window: Option<window::EventThreadWindowEventHook>,
  keyboard: Option<keyboard::EventThreadKeyboardHook>,
  mouse: Option<mouse::EventThreadMouseHook>,
  display: Option<display::EventThreadDisplayHook>,
}

#[derive(thiserror::Error, Debug)]
pub enum RegisterError {
  #[error("this hook is already registered")]
  AlreadyRegistered,
}

#[derive(thiserror::Error, Debug)]
pub enum DispatchError<E> {
  #[error("hook is not registered")]
  NoHook,
  #[error("dispatch error")]
  DispatchError(#[from] tokio::sync::mpsc::error::SendError<E>),
}

impl Hooks {
  pub fn register_window_hook(
    &mut self,
    hook: window::EventThreadWindowEventHook,
  ) -> Result<(), RegisterError> {
    if self.window.is_some() {
      return Err(RegisterError::AlreadyRegistered);
    }
    self.window = Some(hook);
    Ok(())
  }

  pub fn register_keyboard_hook(
    &mut self,
    hook: keyboard::EventThreadKeyboardHook,
  ) -> Result<(), RegisterError> {
    if self.keyboard.is_some() {
      return Err(RegisterError::AlreadyRegistered);
    }
    self.keyboard = Some(hook);
    Ok(())
  }

  pub fn register_mouse_hook(
    &mut self,
    hook: mouse::EventThreadMouseHook,
  ) -> Result<(), RegisterError> {
    if self.mouse.is_some() {
      return Err(RegisterError::AlreadyRegistered);
    }
    self.mouse = Some(hook);
    Ok(())
  }

  pub fn register_display_hook(
    &mut self,
    hook: display::EventThreadDisplayHook,
  ) -> Result<(), RegisterError> {
    if self.display.is_some() {
      return Err(RegisterError::AlreadyRegistered);
    }
    self.display = Some(hook);
    Ok(())
  }

  pub fn dispatch_window_event(
    &self,
    event: WindowEvent,
  ) -> Result<(), DispatchError<WindowEvent>> {
    if let Some(hook) = &self.window {
      hook.dispatch(event)?;
      Ok(())
    } else {
      Err(DispatchError::NoHook)
    }
  }
}
