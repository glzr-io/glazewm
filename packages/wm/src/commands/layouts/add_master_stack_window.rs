use anyhow::Context;
use wm_common::{TilingDirection, WindowState};

use crate::{
  commands::container::attach_container,
  models::{Container, SplitContainer, Workspace},
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn add_master_stack_window(
  target_parent: Option<Container>,
  window_state: &WindowState,
  target_workspace: Workspace,
  target_container: Container,
  config: &UserConfig,
  state: &mut WmState,
) -> anyhow::Result<(Container, usize)> {
  let child_c = target_workspace.child_count();
  let child_count = target_workspace.children().iter().count();
  assert_eq!(child_c, child_count);
  if child_count == 0 {
    Ok((target_workspace.clone().into(), 0))
  } else if child_count == 1 {
    // Create a vertical split container for the stack
    let stack_container = SplitContainer::new(
      TilingDirection::Vertical,
      config.value.gaps.clone(),
    );
    {
      let master_containers = target_workspace.borrow_children_mut();
      let master_container = master_containers.front().unwrap();

      // master_container
      //   .as_tiling_container()?
      //   .set_tiling_size(master_ratio);

      state
        .pending_sync
        .queue_container_to_redraw(master_container.clone());
    }

    attach_container(
      &stack_container.clone().into(),
      &target_workspace.clone().into(),
      None,
    )?;
    Ok((stack_container.clone().into(), 0))
  } else if child_count == 2 {
    let children = target_workspace.children();
    let stack_container = children.back().context("No children.")?;

    Ok((stack_container.clone(), 0))
  } else {
    assert!(false);
    // If there are no children, just append to the workspace.
    Ok((target_workspace.clone().into(), 0))
  }
}
