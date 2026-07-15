use std::cell::Ref;

use ambassador::delegatable_trait;
use anyhow::Context;
use wm_common::{GapsConfig, TilingDirection, TilingLayout};

use super::{CommonGetters, TilingDirectionGetters, TilingLayoutGetters};
use crate::models::{DirectionContainer, TilingContainer};

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
    let monitor_rect = monitor.native_properties().bounds;
    let gaps_config = self.gaps_config();

    let scale_factor = if gaps_config.scale_with_dpi {
      monitor.native_properties().scale_factor
    } else {
      1.
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

  /// Gets accordion padding in pixels for a given layout axis length.
  fn accordion_padding(&self, axis_length: i32) -> anyhow::Result<i32> {
    let monitor = self.monitor().context("No monitor.")?;
    let gaps_config = self.gaps_config();

    let scale_factor = if gaps_config.scale_with_dpi {
      monitor.native_properties().scale_factor
    } else {
      1.
    };

    Ok(
      gaps_config
        .accordion_padding
        .to_px(axis_length, Some(scale_factor)),
    )
  }

  /// Gets the container to resize when resizing a tiling window.
  fn container_to_resize(
    &self,
    is_width_resize: bool,
  ) -> anyhow::Result<Option<TilingContainer>> {
    let requested_direction = if is_width_resize {
      TilingDirection::Horizontal
    } else {
      TilingDirection::Vertical
    };

    let mut candidate = self.as_tiling_container()?;

    loop {
      let parent = candidate
        .parent()
        .and_then(|parent| parent.as_direction_container().ok())
        .context("No parent.")?;

      let controls_requested_axis = parent.tiling_layout()
        == TilingLayout::Tiles
        && parent.tiling_direction() == requested_direction;

      if controls_requested_axis && candidate.tiling_siblings().count() > 0
      {
        return Ok(Some(candidate));
      }

      candidate = match parent {
        DirectionContainer::Split(split) => split.into(),
        DirectionContainer::Workspace(_) => return Ok(None),
      };
    }
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

#[cfg(test)]
mod tests {
  use wm_common::{TilingDirection, TilingLayout};

  use crate::{
    models::{SplitContainer, TilingWindow, Workspace},
    traits::{CommonGetters, TilingLayoutGetters, TilingSizeGetters},
  };

  #[test]
  fn accordion_at_workspace_root_cannot_resize_children() {
    let window = TilingWindow::mock().call();
    let workspace = Workspace::mock()
      .tiling_containers(vec![window.clone().into()])
      .call();
    workspace.set_tiling_layout(TilingLayout::Accordion);

    assert!(window
      .container_to_resize(true)
      .expect("Resize target lookup should succeed.")
      .is_none());
    assert!(window
      .container_to_resize(false)
      .expect("Resize target lookup should succeed.")
      .is_none());
  }

  #[test]
  fn nested_accordion_resizes_as_a_group() {
    let window = TilingWindow::mock().call();
    let accordion = SplitContainer::mock()
      .tiling_containers(vec![window.clone().into()])
      .call();
    accordion.set_tiling_layout(TilingLayout::Accordion);

    let _workspace = Workspace::mock()
      .tiling_direction(TilingDirection::Horizontal)
      .tiling_containers(vec![
        accordion.clone().into(),
        TilingWindow::mock().call().into(),
      ])
      .call();

    let resize_target = window
      .container_to_resize(true)
      .expect("Resize target lookup should succeed.")
      .expect("Accordion group should be resizable.");

    assert_eq!(resize_target.id(), accordion.id());
    assert!(window
      .container_to_resize(false)
      .expect("Resize target lookup should succeed.")
      .is_none());
  }

  #[test]
  fn resize_skips_hidden_share_above_nested_tiles() {
    let window = TilingWindow::mock().call();
    let inner_tiles = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![
        window.clone().into(),
        TilingWindow::mock().call().into(),
      ])
      .call();

    let accordion = SplitContainer::mock()
      .tiling_containers(vec![
        inner_tiles.into(),
        TilingWindow::mock().call().into(),
      ])
      .call();
    accordion.set_tiling_layout(TilingLayout::Accordion);

    let _workspace = Workspace::mock()
      .tiling_direction(TilingDirection::Horizontal)
      .tiling_containers(vec![
        accordion.clone().into(),
        TilingWindow::mock().call().into(),
      ])
      .call();

    let width_target = window
      .container_to_resize(true)
      .expect("Resize target lookup should succeed.")
      .expect("Outer accordion group should be resizable.");
    let height_target = window
      .container_to_resize(false)
      .expect("Resize target lookup should succeed.")
      .expect("Inner tiles child should be resizable.");

    assert_eq!(width_target.id(), accordion.id());
    assert_eq!(height_target.id(), window.id());
  }
}
