use wm_common::{Memo, Rect};

#[derive(Clone, Debug)]
pub struct NativeWindow {
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
  pub fn new(handle: isize, dispatcher: EventLoopDispatcher) -> Self {
    Self {
      dispatcher,
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
    self.dispatcher.run(self.handle, |window| window.title())
  }

  pub fn invalidate_title(&self) -> anyhow::Result<String> {
    self.dispatcher.run(self.handle, |window| window.title())
  }
}

impl PartialEq for NativeWindow {
  fn eq(&self, other: &Self) -> bool {
    self.handle == other.handle
  }
}

impl Eq for NativeWindow {}
