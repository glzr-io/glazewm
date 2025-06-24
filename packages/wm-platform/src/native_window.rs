use std::time::Duration;

use anyhow::{bail, Context};
use tokio::task;
use tracing::warn;
use wm_common::{
  Color, CornerStyle, Delta, HideMethod, LengthValue, Memo, OpacityValue,
  Rect, RectDelta, WindowState,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ZOrder {
  Normal,
  AfterWindow(isize),
  Top,
  TopMost,
}

#[derive(Clone, Debug)]
pub struct NativeWindow {
  pub handle: isize,
  inner: platform_impl::NativeWindow,
  title: Memo<String>,
  process_name: Memo<String>,
  #[cfg(windows)]
  class_name: Memo<String>,
  frame_position: Memo<Rect>,
  border_position: Memo<Rect>,
  is_minimized: Memo<bool>,
  is_maximized: Memo<bool>,
}

impl NativeWindow {
  /// Creates a new `NativeWindow` instance with the given window handle.
  #[must_use]
  pub fn new(handle: isize, event_loop: &EventLoop) -> Self {
    let inner = platform_impl::NativeWindow::new(handle, event_loop);

    Self {
      handle,
      inner,
      title: Memo::new(),
      process_name: Memo::new(),
      #[cfg(windows)]
      class_name: Memo::new(),
      frame_position: Memo::new(),
      border_position: Memo::new(),
      is_minimized: Memo::new(),
      is_maximized: Memo::new(),
    }
  }

  /// Gets the window's title. If the window is invalid, returns an empty
  /// string.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn title(&self) -> anyhow::Result<String> {
    self.title.get_or_init(self.inner.title, self)
  }

  /// Updates the cached window title.
  pub fn invalidate_title(&self) -> anyhow::Result<String> {
    self.title.update(self.inner.title, self)
  }

  /// Gets the process name associated with the window.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn process_name(&self) -> anyhow::Result<String> {
    self.process_name.get_or_init(self.inner.process_name, self)
  }

  /// Gets the class name of the window.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn class_name(&self) -> anyhow::Result<String> {
    self.class_name.get_or_init(self.inner.class_name, self)
  }

  /// Whether the window is actually visible.
  pub fn is_visible(&self) -> anyhow::Result<bool> {
    todo!()
  }

  pub fn cleanup(&self) {
    todo!()
  }
}

impl PartialEq for NativeWindow {
  fn eq(&self, other: &Self) -> bool {
    self.handle == other.handle
  }
}

impl Eq for NativeWindow {}
