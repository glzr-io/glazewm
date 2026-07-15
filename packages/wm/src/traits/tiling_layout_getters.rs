use ambassador::delegatable_trait;
use wm_common::TilingLayout;

use super::CommonGetters;

/// Accessors for the layout used by a tiling parent container.
#[delegatable_trait]
pub trait TilingLayoutGetters: CommonGetters {
  fn tiling_layout(&self) -> TilingLayout;

  fn set_tiling_layout(&self, tiling_layout: TilingLayout);
}

/// Implements the `TilingLayoutGetters` trait for a given struct.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with a `tiling_layout` field.
#[macro_export]
macro_rules! impl_tiling_layout_getters {
  ($struct_name:ident) => {
    impl TilingLayoutGetters for $struct_name {
      fn tiling_layout(&self) -> TilingLayout {
        self.0.borrow().tiling_layout.clone()
      }

      fn set_tiling_layout(&self, tiling_layout: TilingLayout) {
        self.0.borrow_mut().tiling_layout = tiling_layout;
      }
    }
  };
}
