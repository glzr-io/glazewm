use super::WindowHandle;

pub struct NativeWindow {
  pub handle: WindowHandle,
  pub title: String,
  pub process_name: String,
  pub class_name: String,
}

impl NativeWindow {
  pub fn new(handle: WindowHandle) -> Self {
    todo!()
  }

  pub fn is_visible(&self) -> bool {
    todo!()
  }

  pub fn is_manageable(&self) -> bool {
    todo!()
  }

  pub fn is_minimized(&self) -> bool {
    todo!()
  }

  pub fn is_maximized(&self) -> bool {
    todo!()
  }

  pub fn is_resizable(&self) -> bool {
    todo!()
  }

  pub fn is_app_bar(&self) -> bool {
    todo!()
  }

  fn window_styles(&self) -> Vec<WindowStyle> {
    todo!()
  }

  fn window_styles_ex(&self) -> Vec<WindowStyleEx> {
    todo!()
  }
}
