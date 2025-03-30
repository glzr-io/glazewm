use wm_common::{TilingDirection, WindowState};

use crate::{
  commands::container::attach_container,
  models::{Container, SplitContainer, Workspace},
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn add_grid_window(
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

  let current_container = target_workspace.clone();
  let mut result = None;
  current_container.children().iter().for_each(|container| {
    if container.child_count() < child_count {
      result = Some((container.clone().into(), 0));
      return;
    }
  });

  if let Some(result) = result {
    return Ok(result);
  }

  let vert_container = SplitContainer::new(
    TilingDirection::Vertical,
    config.value.gaps.clone(),
  );
  {
    let master_containers = target_workspace.borrow_children_mut();
    state
      .pending_sync
      .queue_containers_to_redraw(master_containers.clone());
  }

  attach_container(
    &vert_container.clone().into(),
    &target_workspace.clone().into(),
    None,
  )?;
  Ok((vert_container.clone().into(), 0))
}
