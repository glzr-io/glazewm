use anyhow::Context;

use crate::{
  common::{Direction, TilingDirection},
  containers::{
    commands::{move_container_within_tree, toggle_tiling_direction},
    traits::{CommonGetters, PositionGetters, TilingDirectionGetters},
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

fn tiling_sibling_in_direction(
  window: TilingWindow,
  direction: &Direction,
) -> Option<TilingContainer> {
  match direction {
    Direction::Up | Direction::Left => window
      .prev_siblings()
      .find_map(|c| c.as_tiling_container().ok()),
    _ => window
      .next_siblings()
      .find_map(|c| c.as_tiling_container().ok()),
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
        )?;

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
      _ => workspace.child_count() - 1,
    };

    move_container_within_tree(
      window_to_move.into(),
      workspace.clone().into(),
      target_index,
    )?;

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
  let workspace = window_to_move.workspace().context("No workspace.")?;

  toggle_tiling_direction(workspace.clone().into(), state, config)?;
  state.containers_to_redraw.push(workspace.into());

  if let Some(sibling) =
    tiling_sibling_in_direction(window_to_move.clone(), direction)
  {
    move_to_sibling_container(window_to_move, sibling, direction, state)?;
  }

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

  // Move the window into the container above.
  let target_index = match direction {
    Direction::Up | Direction::Left => window_ancestor.index(),
    _ => window_ancestor.index() + 1,
  };

  move_container_within_tree(
    window_to_move.clone().into(),
    target_ancestor.clone().into(),
    target_index,
  )?;

  state.containers_to_redraw.push(target_ancestor.into());

  Ok(())
}

fn move_floating_window(
  window_to_move: NonTilingWindow,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  todo!()
}
