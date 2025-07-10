mod display;
mod keyboard;
mod mouse;
mod window;

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
}
