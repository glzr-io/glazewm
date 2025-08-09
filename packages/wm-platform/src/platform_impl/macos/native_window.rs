use objc2_core_foundation::{CFBoolean, CFRetained, CFString, CGSize};
use wm_common::{Memo, Rect};

use crate::platform_impl::{
  AXUIElement, AXUIElementExt, AXValue, EventLoopDispatcher, MainThreadRef,
};

#[derive(Clone, Debug)]
pub struct NativeWindow {
  element: MainThreadRef<CFRetained<AXUIElement>>,
  dispatcher: EventLoopDispatcher,
  pub handle: isize,
  title: Memo<String>,
  process_name: Memo<String>,
  class_name: Memo<String>,
  frame_position: Memo<Rect>,
  border_position: Memo<Rect>,
  is_minimized: Memo<bool>,
  is_maximized: Memo<bool>,
}

impl NativeWindow {
  /// Creates a new `NativeWindow` instance with the given window handle.
  #[must_use]
  pub fn new(
    handle: isize,
    dispatcher: EventLoopDispatcher,
    element: MainThreadRef<CFRetained<AXUIElement>>,
  ) -> Self {
    Self {
      dispatcher,
      element,
      handle,
      title: Memo::new(),
      process_name: Memo::new(),
      class_name: Memo::new(),
      frame_position: Memo::new(),
      border_position: Memo::new(),
      is_minimized: Memo::new(),
      is_maximized: Memo::new(),
    }
  }

  pub fn title(&self) -> anyhow::Result<String> {
    self.title.get_or_init(Self::updated_title, self)
  }

  pub fn invalidate_title(&self) -> anyhow::Result<String> {
    self.title.update(Self::updated_title, self)
  }

  fn updated_title(&self) -> anyhow::Result<String> {
    self.element.with(|el| {
      el.get_attribute::<CFString>("AXTitle")
        .map(|r| r.to_string())
    })?
  }

  pub fn process_name(&self) -> anyhow::Result<String> {
    // AX has AXProcessIdentifier; getting name requires more hops. Stub.
    Ok(String::new())
  }

  pub fn class_name(&self) -> anyhow::Result<String> {
    // AXRole / AXSubrole might serve as class-like identifiers.
    self.element.with(|el| {
      el.get_attribute::<CFString>("AXRole")
        .map(|r| r.to_string())
    })?
  }

  pub fn is_visible(&self) -> anyhow::Result<bool> {
    // Heuristic: visible if not minimized.
    let minimized = self
      .element
      .with(|el| el.get_attribute::<CFBoolean>("AXMinimized"))
      .and_then(|r| r.map(|cf_bool| cf_bool.value()))?;
    Ok(!minimized)
  }

  pub fn resize(&self, size: Rect) -> anyhow::Result<()> {
    let width = size.width() as f64;
    let height = size.height() as f64;

    self.element.with(move |el| -> anyhow::Result<()> {
      let ax_size = CGSize::new(width, height);
      let ax_value = AXValue::new(&ax_size)?;
      el.set_attribute("AXSize", &ax_value)
    })??;

    Ok(())
  }

  /// Whether the window is minimized.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn is_minimized(&self) -> anyhow::Result<bool> {
    self
      .is_minimized
      .get_or_init(Self::updated_is_minimized, self)
  }

  /// Updates the cached minimized status.
  pub fn invalidate_is_minimized(&self) -> anyhow::Result<bool> {
    self.is_minimized.update(Self::updated_is_minimized, self)
  }

  /// Whether the window is minimized.
  #[allow(clippy::unnecessary_wraps)]
  fn updated_is_minimized(&self) -> anyhow::Result<bool> {
    self
      .element
      .with(|el| el.get_attribute::<CFBoolean>("AXMinimized"))
      .and_then(|r| r.map(|cf_bool| cf_bool.value()))
  }

  pub fn cleanup(&self) {
    let _ = self.invalidate_title();
  }
}

impl From<NativeWindow> for crate::NativeWindow {
  fn from(window: NativeWindow) -> Self {
    crate::NativeWindow { inner: window }
  }
}
