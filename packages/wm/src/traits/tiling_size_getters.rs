use std::cell::Ref;

use ambassador::delegatable_trait;
use anyhow::Context;
use wm_common::{GapsConfig, TilingDirection};

use super::{CommonGetters, PositionGetters, TilingDirectionGetters};
use crate::models::{Container, DirectionContainer, TilingContainer};

pub const MIN_TILING_SIZE: f32 = 0.01;

#[delegatable_trait]
pub trait TilingSizeGetters: CommonGetters {
  fn tiling_size(&self) -> f32;

  fn set_tiling_size(&self, tiling_size: f32);

  fn gaps_config(&self) -> Ref<'_, GapsConfig>;

  fn set_gaps_config(&self, gaps_config: GapsConfig);

  /// Gets the horizontal and vertical gaps between windows in pixels.
  fn inner_gaps(&self) -> anyhow::Result<(i32, i32)> {
    let monitor = self.monitor().context("No monitor.")?;
    let monitor_rect = monitor.to_rect()?;
    let gaps_config = self.gaps_config();

    let scale_factor = match gaps_config.scale_with_dpi {
      true => monitor.native().scale_factor()?,
      false => 1.,
    };

    Ok((
      gaps_config
        .inner_gap
        .to_px(monitor_rect.height(), Some(scale_factor)),
      gaps_config
        .inner_gap
        .to_px(monitor_rect.width(), Some(scale_factor)),
    ))
  }

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

      fn gaps_config(&self) -> Ref<'_, GapsConfig> {
        Ref::map(self.0.borrow(), |inner| &inner.gaps_config)
      }

      fn set_gaps_config(&self, gaps_config: GapsConfig) {
        self.0.borrow_mut().gaps_config = gaps_config;
      }
    }
  };
}
