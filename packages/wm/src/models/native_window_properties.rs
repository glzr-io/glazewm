use wm_platform::{NativeWindow, NativeWindowWindowsExt, Rect, RectDelta};

#[derive(Debug, Clone)]
pub struct NativeWindowProperties {
  pub title: String,
  pub class_name: String,
  pub process_name: String,
  pub frame: Rect,
  pub is_minimized: bool,
  pub is_maximized: bool,
  pub is_resizable: bool,
  pub shadow_borders: RectDelta,
}

impl TryFrom<&NativeWindow> for NativeWindowProperties {
  type Error = anyhow::Error;

  fn try_from(native_window: &NativeWindow) -> Result<Self, Self::Error> {
    Ok(Self {
      title: native_window.title()?,
      class_name: native_window.class_name()?,
      process_name: native_window.process_name()?,
      frame: native_window.frame()?,
      is_minimized: native_window.is_minimized()?,
      is_maximized: native_window.is_maximized()?,
      is_resizable: native_window.is_resizable()?,
      shadow_borders: native_window.shadow_borders()?,
    })
  }
}
