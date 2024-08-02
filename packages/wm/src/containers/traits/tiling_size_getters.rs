use ambassador::delegatable_trait;
use anyhow::Context;

use super::{CommonGetters, TilingDirectionGetters};
use crate::{
  common::{LengthValue, TilingDirection},
  containers::{Container, DirectionContainer, TilingContainer},
};

pub const MIN_TILING_SIZE: f32 = 0.01;

#[delegatable_trait]
pub trait TilingSizeGetters: CommonGetters {
  fn tiling_size(&self) -> f32;

  fn set_tiling_size(&self, tiling_size: f32);

  fn inner_gap(&self) -> LengthValue;

  fn set_inner_gap(&self, inner_gap: LengthValue);

  /// Gets the container to resize when resizing a tiling window.
  fn container_to_resize(
    &self,
    is_width_resize: bool,
  ) -> anyhow::Result<Option<TilingContainer>> {
    let parent = self.direction_container().context("No parent.")?;

    let tiling_direction = parent.tiling_direction();

    // Whether the resize is in the inverse of its tiling direction.
    let is_inverse_resize = match tiling_direction {
      TilingDirection::Horizontal => !is_width_resize,
      TilingDirection::Vertical => is_width_resize,
    };

    let container_to_resize = match is_inverse_resize {
      true => match parent {
        // Prevent workspaces from being resized.
        DirectionContainer::Split(parent) => Some(parent.into()),
        _ => None,
      },
      false => {
        let grandparent = parent.parent().context("No grandparent.")?;

        match self.tiling_siblings().count() > 0 {
          // Window can only be resized if it has siblings.
          true => Some(self.as_tiling_container()?),
          // Resize grandparent in layouts like H[1 V[2 H[3]]], where
          // container 3 is resized horizontally.
          false => match grandparent {
            Container::Split(grandparent) => Some(grandparent.into()),
            _ => None,
          },
        }
      }
    };

    Ok(container_to_resize)
  }
}

/// Implements the `TilingSizeGetters` trait for a given struct.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with a `tiling_size` field.
#[macro_export]
macro_rules! impl_tiling_size_getters {
  ($struct_name:ident) => {
    impl TilingSizeGetters for $struct_name {
      fn tiling_size(&self) -> f32 {
        self.0.borrow().tiling_size
      }

      fn set_tiling_size(&self, tiling_size: f32) {
        self.0.borrow_mut().tiling_size = tiling_size;
      }

      fn inner_gap(&self) -> LengthValue {
        self.0.borrow().inner_gap.clone()
      }

      fn set_inner_gap(&self, inner_gap: LengthValue) {
        self.0.borrow_mut().inner_gap = inner_gap;
      }
    }
  };
}
