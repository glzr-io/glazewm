use enum_dispatch::enum_dispatch;

use crate::{
  common::{platform::NativeWindow, DisplayState, RectDelta},
  containers::WindowContainer,
  windows::WindowState,
};

#[enum_dispatch]
pub trait WindowBehavior {
  fn state(&self) -> WindowState;

  fn native(&self) -> NativeWindow;

  fn border_delta(&self) -> RectDelta;

  fn display_state(&self) -> DisplayState;

  fn set_display_state(&self, display_state: DisplayState) -> ();

  fn has_pending_dpi_adjustment(&self) -> bool;

  fn set_has_pending_dpi_adjustment(
    &self,
    has_pending_dpi_adjustment: bool,
  ) -> ();
}

/// Implements the `WindowBehavior` trait for a given struct.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with a `state`, `native`, and a `display_state` field.
#[macro_export]
macro_rules! impl_window_behavior {
  ($struct_name:ident) => {
    impl WindowBehavior for $struct_name {
      fn state(&self) -> WindowState {
        self.0.borrow().state.clone()
      }

      fn native(&self) -> NativeWindow {
        self.0.borrow().native.clone()
      }

      fn border_delta(&self) -> RectDelta {
        self.0.borrow().border_delta.clone()
      }

      fn display_state(&self) -> DisplayState {
        self.0.borrow().display_state.clone()
      }

      fn set_display_state(&self, display_state: DisplayState) {
        self.0.borrow_mut().display_state = display_state;
      }

      fn has_pending_dpi_adjustment(&self) -> bool {
        self.0.borrow().has_pending_dpi_adjustment
      }

      fn set_has_pending_dpi_adjustment(
        &self,
        has_pending_dpi_adjustment: bool,
      ) {
        self.0.borrow_mut().has_pending_dpi_adjustment =
          has_pending_dpi_adjustment;
      }
    }
  };
}
