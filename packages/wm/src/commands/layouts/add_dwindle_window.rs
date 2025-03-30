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
  // Create a TilingWindow from NativeWindow.
  let border_delta = RectDelta::new(
    LengthValue::from_px(0),
    LengthValue::from_px(0),
    LengthValue::from_px(0),
    LengthValue::from_px(0),
  );
  let gaps_config = config.value.gaps.clone();
  let new_window = TilingWindow::new(
    None,
    native_window,
    None,
    border_delta,
    Rect {
      left: 0,
      top: 0,
      right: 0,
      bottom: 0,
    },
    false,
    gaps_config,
    Vec::new(),
    None,
  );

  // Setup the initial dwindle layout.
  if target_workspace.child_count() == 0 {
    attach_container(
      &new_window.clone().into(),
      &target_workspace.clone().into(),
      None,
    )?;
    return new_window.as_window_container();
  } else if target_workspace.child_count() == 1 {
    let new_split_container = SplitContainer::new(
      TilingDirection::Vertical,
      config.value.gaps.clone(),
    );
    attach_container(
      &new_window.clone().into(),
      &new_split_container.clone().into(),
      None,
    )?;
    attach_container(
      &new_split_container.clone().into(),
      &target_workspace.clone().into(),
      None,
    )?;
    return new_window.as_window_container();
  }

  // At this point, we have at least 2 windows.
  // This logic inserts the new window after the focused window, shifting
  // all deeper windows in the dwindle tree downwards to make room.
  let target_parent = target_container.parent().context("No parent.")?;
  let child_count = target_parent.child_count();
  let focused_index = target_container.index();

  // Re-organize the containers
  let mut workspace_children = target_parent.borrow_children_mut();

  // The 2nd container is always a SplitContainer, 1st is just a window.
  let mut split_container =
    workspace_children.back().context("No children.")?;
  let mut split_children = split_container.borrow_children_mut();
  let window_to_shift =
    detach_container(*split_children.front().unwrap()).ok();
  attach_container(
    &new_window.clone().into(),
    &split_container.clone().into(),
    None,
  )?;
  // let mut window_to_shift = split_children.pop_front();
  // split_children.insert(0, target_container.clone());
  //
  while let Some(window) = window_to_shift {
    // Get the back container using a temporary reference
    let back = {
      let current_children = &split_children;
      current_children.back().context("No children.")?
    };

    // Clone the container since we need to use it after the borrow ends
    let next_container = back.clone();

    // Get the children of the next container
    let mut next_children = next_container.children();

    if next_children.iter().count() == 1 {
      // Create the new split container
      let new_split_direction = next_container
        .as_direction_container()?
        .tiling_direction()
        .inverse();

      let new_split_container = SplitContainer::new(
        new_split_direction,
        config.value.gaps.clone(),
      );

      // Add the window to the new container and attach
      new_split_container.borrow_children_mut().push_back(window);
      attach_container(
        &new_split_container.clone().into(),
        &next_container.clone().into(),
        None,
      )?;
      break;
    }

    // Shift windows
    let next_window = next_children.pop_front();
    next_children.insert(0, window);

    // Prepare for next iteration
    split_container = &next_container;
    split_children = split_container.children();
    window_to_shift = next_window;
  }
  new_window.as_window_container()
}
// ) -> anyhow::Result<(Container, usize)> {
//   let child_count = target_workspace.child_count();
//
//   if child_count == 0 {
//     Ok((target_workspace.clone().into(), 0))
//   } else if child_count == 1 {
//     // Create a vertical split container for the stack
//     let new_container = SplitContainer::new(
//       TilingDirection::Vertical,
//       config.value.gaps.clone(),
//     );
//     {
//       let master_containers = target_workspace.borrow_children_mut();
//       let only_container = master_containers.front().unwrap();
//       state
//         .pending_sync
//         .queue_container_to_redraw(only_container.clone());
//     }
//     attach_container(
//       &new_container.clone().into(),
//       &target_workspace.clone().into(),
//       None,
//     )?;
//     Ok((new_container.clone().into(), 0))
//   } else if child_count == 2 {
//     let children = target_workspace.borrow_children_mut();
//     let back_container = children.back().context("No children.")?;
//     let back_clone = back_container.clone();
//     drop(children);
//
//     // Now work with the cloned container
//     let mut current_container = back_clone;
//
//     while current_container.children().iter().count() > 1 {
//       // Create a new borrow scope
//       let next_container = {
//         let children = current_container.borrow_children_mut();
//         let back = children.back().context("No children.")?;
//         back.clone() // Clone it so we can drop the borrow
//       };
//       current_container = next_container;
//     }
//
//     let current_child_count =
// current_container.children().iter().count();     if current_child_count
// == 0 {       Ok((target_workspace.clone().into(), 0))
//     } else if current_child_count == 1 {
//       let new_split_direction = current_container
//         .as_direction_container()?
//         .tiling_direction()
//         .inverse();
//       let split_container = SplitContainer::new(
//         new_split_direction,
//         config.value.gaps.clone(),
//       );
//       state
//         .pending_sync
//         .queue_container_to_redraw(current_container.clone());
//       attach_container(
//         &split_container.clone().into(),
//         &current_container.clone().into(),
//         None,
//       )?;
//       Ok((split_container.clone().into(), 0))
//     } else {
//       Err(anyhow::anyhow!("Unexpected child count"))
//     }
//   } else {
//     Err(anyhow::anyhow!("Unexpected child count"))
//   }
// }
