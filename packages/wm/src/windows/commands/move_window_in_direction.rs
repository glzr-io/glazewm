use anyhow::Context;

use crate::{
  common::{Direction, TilingDirection},
  containers::{
    commands::{
      flatten_split_container, move_container_within_tree,
      resize_tiling_container,
    },
    traits::{
      CommonGetters, PositionGetters, TilingDirectionGetters,
      TilingSizeGetters,
    },
    Container, DirectionContainer, SplitContainer, TilingContainer,
    WindowContainer,
  },
  user_config::UserConfig,
  windows::{
    traits::WindowGetters, NonTilingWindow, TilingWindow, WindowState,
  },
  wm_state::WmState,
};

pub fn move_window_in_direction(
  window: WindowContainer,
  direction: &Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  match window {
    WindowContainer::TilingWindow(window) => {
      move_tiling_window(window, direction, state, config)
    }
    WindowContainer::NonTilingWindow(non_tiling_window) => {
      match non_tiling_window.state() {
        WindowState::Floating(_) => {
          move_floating_window(non_tiling_window, direction, state)
        }
        WindowState::Fullscreen(_) => {
          todo!()
        }
        _ => Ok(()),
      }
    }
  }
}

fn move_tiling_window(
  window_to_move: TilingWindow,
  direction: &Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let parent = window_to_move
    .direction_container()
    .context("No direction container.")?;

  let has_matching_tiling_direction = parent.tiling_direction()
    == TilingDirection::from_direction(direction);

  // Attempt to swap or move the window into a sibling container.
  if has_matching_tiling_direction {
    if let Some(sibling) =
      tiling_sibling_in_direction(window_to_move.clone(), direction)
    {
      return move_to_sibling_container(
        window_to_move,
        sibling,
        direction,
        state,
      );
    }
  }

  // Attempt to move the window to workspace in given direction.
  if (has_matching_tiling_direction
    || window_to_move.tiling_siblings().count() == 0)
    && parent.is_workspace()
  {
    return move_to_workspace_in_direction(
      window_to_move,
      direction,
      state,
    );
  }

  // The window cannot be moved within the parent container, so traverse
  // upwards to find an ancestor that has the correct tiling direction.
  let target_ancestor = parent.ancestors().find_map(|ancestor| {
    ancestor.as_direction_container().ok().filter(|ancestor| {
      ancestor.tiling_direction()
        == TilingDirection::from_direction(direction)
    })
  });

  match target_ancestor {
    // If there is no suitable ancestor, then change the tiling direction
    // of the workspace.
    None => change_workspace_tiling_direction(
      window_to_move,
      direction,
      state,
      config,
    ),
    // Otherwise, move the container into the given ancestor. This could
    // simply be the container's direct parent.
    Some(target_ancestor) => insert_into_ancestor(
      window_to_move,
      target_ancestor,
      direction,
      state,
    ),
  }
}

/// Gets the next sibling `TilingWindow` or `SplitContainer` in the given
/// direction.
fn tiling_sibling_in_direction(
  window: TilingWindow,
  direction: &Direction,
) -> Option<TilingContainer> {
  match direction {
    Direction::Up | Direction::Left => window
      .prev_siblings()
      .find_map(|sibling| sibling.as_tiling_container().ok()),
    _ => window
      .next_siblings()
      .find_map(|sibling| sibling.as_tiling_container().ok()),
  }
}

fn move_to_sibling_container(
  window_to_move: TilingWindow,
  target_sibling: TilingContainer,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let parent = window_to_move.parent().context("No parent.")?;

  match target_sibling {
    TilingContainer::TilingWindow(sibling_window) => {
      // Swap the window with sibling in given direction.
      move_container_within_tree(
        window_to_move.clone().into(),
        parent,
        sibling_window.index(),
        state,
      )?;

      state
        .containers_to_redraw
        .extend([sibling_window.into(), window_to_move.into()]);
    }
    TilingContainer::Split(sibling_split) => {
      let sibling_descendant =
        sibling_split.descendant_in_direction(&direction.inverse());

      // Move the window into the sibling split container.
      if let Some(sibling_descendant) = sibling_descendant {
        let target_parent = sibling_descendant
          .direction_container()
          .context("No direction container.")?;

        let has_matching_tiling_direction =
          TilingDirection::from_direction(direction)
            == target_parent.tiling_direction();

        let target_index = match direction {
          Direction::Down | Direction::Right
            if has_matching_tiling_direction =>
          {
            sibling_descendant.index()
          }
          _ => sibling_descendant.index() + 1,
        };

        move_container_within_tree(
          window_to_move.into(),
          target_parent.clone().into(),
          target_index,
          state,
        )?;

        // TODO: Only redraw tiling containers.
        state
          .containers_to_redraw
          .extend([target_parent.into(), parent.into()]);
      }
    }
  };

  Ok(())
}

fn move_to_workspace_in_direction(
  window_to_move: TilingWindow,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let parent = window_to_move.parent().context("No parent.")?;
  let monitor = parent.monitor().context("No monitor.")?;

  let workspace_in_direction = state
    .monitor_in_direction(&monitor, direction)?
    .and_then(|monitor| monitor.displayed_workspace());

  if let Some(workspace) = workspace_in_direction {
    // Since the window is crossing monitors, adjustments might need to be
    // made because of DPI.
    if monitor.has_dpi_difference(&workspace.clone().into())? {
      window_to_move.set_has_pending_dpi_adjustment(true);
    }

    // Update floating placement since the window has to cross monitors.
    window_to_move.set_floating_placement(
      window_to_move
        .floating_placement()
        .translate_to_center(&workspace.to_rect()?),
    );

    let target_index = match direction {
      Direction::Down | Direction::Right => 0,
      _ => workspace.child_count(),
    };

    move_container_within_tree(
      window_to_move.into(),
      workspace.clone().into(),
      target_index,
      state,
    )?;

    // TODO: Only redraw tiling containers.
    state
      .containers_to_redraw
      .extend([workspace.into(), parent.into()]);
  };

  Ok(())
}

fn change_workspace_tiling_direction(
  window_to_move: TilingWindow,
  direction: &Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // 1. Change the tiling direction of the workspace.
  // 2. All other containers are wrapped in a split container.
  // 3. Split container is resized to be 50% (and window is also 50%).
  // 4. Depending on direction, place window before/after split container.

  let workspace = window_to_move.workspace().context("No workspace.")?;
  let parent = window_to_move.parent().context("No parent.")?;

  if let Some(split_parent) = parent.as_split().cloned() {
    if split_parent.child_count() == 1 {
      flatten_split_container(split_parent)?;
    }
  }

  // Create a new split container to wrap the window's siblings.
  let split_container = SplitContainer::new(
    workspace.tiling_direction(),
    config.value.gaps.inner_gap.clone(),
  );

  wrap_in_split_container(
    split_container,
    parent.clone(),
    window_to_move.clone().tiling_siblings().collect(),
  )?;

  // Invert the tiling direction of the workspace.
  workspace.set_tiling_direction(workspace.tiling_direction().inverse());

  let target_index = match direction {
    Direction::Left | Direction::Up => 0,
    _ => workspace.child_count(),
  };

  move_container_within_tree(
    window_to_move.clone().into(),
    parent,
    target_index,
    state,
  )?;

  resize_tiling_container(&window_to_move.into(), 0.5);

  state
    .containers_to_redraw
    .extend(workspace.tiling_children().map(Into::into));

  Ok(())
}

fn wrap_in_split_container(
  split_container: SplitContainer,
  target_parent: Container,
  target_children: Vec<TilingContainer>,
) -> anyhow::Result<()> {
  let mut focus_indices = target_children
    .iter()
    .map(|child| child.focus_index())
    .collect::<Vec<_>>();

  // Sort the focus indices in ascending order.
  focus_indices.sort();

  let starting_index = target_children
    .iter()
    .min_by_key(|&child| child.index())
    .map(|child| child.index())
    .context("Failed to get starting index.")?;

  target_parent
    .borrow_children_mut()
    .insert(starting_index, split_container.clone().into());

  let starting_focus_index = *focus_indices
    .iter()
    .min()
    .context("Failed to get starting focus index.")?;

  target_parent
    .borrow_child_focus_order_mut()
    .insert(starting_focus_index, split_container.id());

  // Get the total tiling size amongst all children.
  let total_tiling_size = target_children
    .iter()
    .map(|child| child.tiling_size())
    .sum::<f32>();

  *split_container.borrow_parent_mut() = Some(target_parent.clone());
  split_container.set_tiling_size(total_tiling_size);

  // Move the children from their original parent to the split container.
  for target_child in target_children.iter() {
    *target_child.borrow_parent_mut() =
      Some(split_container.clone().into());

    split_container
      .borrow_children_mut()
      .push_front(target_child.clone().into());

    split_container
      .borrow_child_focus_order_mut()
      .push_front(target_child.id());

    target_parent
      .borrow_children_mut()
      .retain(|child| child != &target_child.clone().into());

    target_parent
      .borrow_child_focus_order_mut()
      .retain(|id| id != &target_child.id());

    // Scale the tiling size to the new split container.
    target_child
      .set_tiling_size(target_child.tiling_size() / total_tiling_size);
  }

  // TODO: Need to adjust focus order of split container.

  Ok(())
}

fn insert_into_ancestor(
  window_to_move: TilingWindow,
  target_ancestor: DirectionContainer,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // Traverse upwards to find container whose parent is the target
  // ancestor. Then, depending on the direction, insert before or after
  // that container.
  let window_ancestor = window_to_move
    .ancestors()
    .find(|container| {
      container
        .parent()
        .map_or(false, |parent| parent == target_ancestor.clone().into())
    })
    .context("Window ancestor not found.")?;

  let target_index = match direction {
    Direction::Up | Direction::Left => window_ancestor.index(),
    _ => window_ancestor.index() + 1,
  };

  // Move the window into the container above.
  move_container_within_tree(
    window_to_move.clone().into(),
    target_ancestor.clone().into(),
    target_index,
    state,
  )?;

  state
    .containers_to_redraw
    .extend(target_ancestor.tiling_children().map(Into::into));

  Ok(())
}

fn move_floating_window(
  window_to_move: NonTilingWindow,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  todo!()
}
