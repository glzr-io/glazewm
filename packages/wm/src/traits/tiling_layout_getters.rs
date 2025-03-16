use ambassador::delegatable_trait;

use super::CommonGetters;
use crate::models::TilingLayout;

#[delegatable_trait]
pub trait TilingLayoutGetters: CommonGetters {
  fn tiling_layout(&self) -> TilingLayout;
}

#[macro_export]
macro_rules! impl_tiling_layout_getters {
  ($struct_name:ident) => {
    impl TilingLayoutGetters for $struct_name {
      fn tiling_layout(&self) -> TilingLayout {
        self.0.borrow().tiling_layout.clone()
      }
    }
  };
}
