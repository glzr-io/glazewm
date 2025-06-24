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
  inner: platform_impl::NativeWindow,
}

impl NativeWindow {
  /// Creates a new `NativeWindow` instance with the given window handle.
  #[must_use]
  pub fn new(handle: isize, event_loop: &EventLoop) -> Self {
    let inner = platform_impl::NativeWindow::new(handle, event_loop);
    Self { inner }
  }

  /// Gets the window's title. If the window is invalid, returns an empty
  /// string.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn title(&self) -> anyhow::Result<String> {
    self.inner.title()
  }

  /// Updates the cached window title.
  pub fn invalidate_title(&self) -> anyhow::Result<String> {
    self.inner.invalidate_title()
  }

  /// Gets the process name associated with the window.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn process_name(&self) -> anyhow::Result<String> {
    self.inner.process_name()
  }

  /// Gets the class name of the window.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn class_name(&self) -> anyhow::Result<String> {
    self.inner.class_name()
  }

  /// Whether the window is actually visible.
  pub fn is_visible(&self) -> anyhow::Result<bool> {
    self.inner.is_visible()
  }

  pub fn cleanup(&self) {
    self.inner.cleanup()
  }
}

impl PartialEq for NativeWindow {
  fn eq(&self, other: &Self) -> bool {
    self.handle == other.handle
  }
}

impl Eq for NativeWindow {}
