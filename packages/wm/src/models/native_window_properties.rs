use wm_platform::{NativeWindow, Rect};
#[cfg(target_os = "windows")]
use wm_platform::{NativeWindowWindowsExt, RectDelta};

#[derive(Debug, Clone)]
pub struct NativeWindowProperties {
  pub title: String,
  #[cfg(target_os = "windows")]
  pub class_name: String,
  pub process_name: String,
  pub frame: Rect,
  /// The window's original position/size before GlazeWM managed it.
  /// Used to restore windows to their initial state when the WM exits.
  pub original_frame: Rect,
  pub is_minimized: bool,
  pub is_maximized: bool,
  /// Whether the window was originally maximized before GlazeWM managed it.
  /// Used to restore maximized state when the WM exits.
  pub original_is_maximized: bool,
  pub is_resizable: bool,
  #[cfg(target_os = "windows")]
  pub shadow_borders: RectDelta,
}

impl TryFrom<&NativeWindow> for NativeWindowProperties {
  type Error = anyhow::Error;

  fn try_from(native_window: &NativeWindow) -> Result<Self, Self::Error> {
    let frame = native_window.frame()?;

    Ok(Self {
      title: native_window.title()?,
      #[cfg(target_os = "windows")]
      class_name: native_window.class_name()?,
      process_name: native_window.process_name()?,
      frame: frame.clone(),
      original_frame: frame,
      is_minimized: native_window.is_minimized()?,
      is_maximized: native_window.is_maximized()?,
      original_is_maximized: native_window.is_maximized()?,
      is_resizable: native_window.is_resizable()?,
      #[cfg(target_os = "windows")]
      shadow_borders: native_window.shadow_borders()?,
    })
  }
}
