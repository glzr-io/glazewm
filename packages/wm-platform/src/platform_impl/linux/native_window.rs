use smithay::desktop::Window;

pub type WindowHandle<'a> = &'a smithay::desktop::Window;

#[derive(Debug, Clone)]
pub struct NativeWindow {
  inner: Window,
}

impl NativeWindow {
  pub fn new(inner: Window) -> Self {
    Self { inner }
  }

  pub fn handle(&self) -> crate::WindowHandle {
    &self.inner
  }
}
