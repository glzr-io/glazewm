use anyhow::Context;
use wm_common::{
  LengthValue, Rect, RectDelta, TilingDirection, WindowState,
};
use wm_platform::NativeWindow;

use crate::{
  commands::container::{attach_container, detach_container},
  models::{
    Container, SplitContainer, TilingWindow, WindowContainer, Workspace,
  },
  traits::{CommonGetters, TilingDirectionGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn add_dwindle_window(
  native_window: NativeWindow,
  target_parent: Option<Container>,
  window_state: &WindowState,
  target_workspace: Workspace,
  target_container: Container,
  config: &UserConfig,
  state: &mut WmState,
) -> anyhow::Result<WindowContainer> {
  let new_window = TilingWindow::new(
    None,
    native_window,
    None,
    RectDelta::new(
      LengthValue::from_px(0),
      LengthValue::from_px(0),
      LengthValue::from_px(0),
      LengthValue::from_px(0),
    ),
    Rect {
      left: 0,
      top: 0,
      right: 0,
      bottom: 0,
    },
    false,
    config.value.gaps.clone(),
    Vec::new(),
    None,
  );

  // Get starting point - use target's parent if it exists
  let start_container = if target_workspace.child_count() <= 1 {
    target_workspace.clone().into()
  } else {
    target_container.parent().context("No parent.")?
  };

  let mut current = start_container;
  let mut window = new_window.clone().into();

  loop {
    if current.child_count() <= 1 {
      if current.child_count() == 0 {
        attach_container(&window, &current, Some(0))?;
        break;
      }

      let direction = if current.as_workspace().is_some() {
        TilingDirection::Vertical
      } else {
        current
          .as_direction_container()?
          .tiling_direction()
          .inverse()
      };

      let new_split =
        SplitContainer::new(direction, config.value.gaps.clone());
      attach_container(&window, &new_split.clone().into(), Some(0))?;
      attach_container(&new_split.clone().into(), &current, Some(1))?;
      break;
    }

    // Get existing split and its window
    let next_split = current.borrow_children().get(1).unwrap().clone();
    let next_window = next_split.borrow_children().get(0).unwrap().clone();

    // Swap windows
    let detached = detach_container(next_window, true)?;
    attach_container(&window, &next_split.clone().into(), Some(0))?;

    // Move to next container
    current = next_split;
    window = detached;
  }

  new_window.as_window_container()
}
