use wm_platform::Rect;

#[derive(Debug, Clone)]
pub struct NativeWindowProperties {
  pub title: String,
  pub class_name: String,
  pub process_name: String,
  pub frame: Rect,
  pub is_visible: bool,
  pub is_minimized: bool,
  pub is_maximized: bool,
  pub is_fullscreen: bool,
}
