use anyhow::Context;
use wm_common::{Direction, Rect, TilingDirection, WindowState};

use crate::{
  commands::container::{
    flatten_child_split_containers, flatten_split_container,
    move_container_within_tree, resize_tiling_container,
    wrap_in_split_container,
  },
  models::{
    DirectionContainer, Monitor, NonTilingWindow, SplitContainer,
    TilingContainer, TilingWindow, WindowContainer,
  },
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters, WindowGetters,
  },
  user_config::UserConfig,
  wm_state::WmState,
};

/// The distance in pixels to snap the window to the monitor's edge.
const SNAP_DISTANCE: i32 = 15;

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
        WindowState::Fullscreen(_) => move_to_workspace_in_direction(
          &non_tiling_window.into(),
          direction,
          state,
        ),
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
  // Flatten the parent split container if it only contains the window.
  if let Some(split_parent) = window_to_move
    .parent()
    .and_then(|parent| parent.as_split().cloned())
  {
    if split_parent.child_count() == 1 {
      flatten_split_container(split_parent)?;
    }
  }

  let parent = window_to_move
    .direction_container()
    .context("No direction container.")?;

  let has_matching_tiling_direction = parent.tiling_direction()
    == TilingDirection::from_direction(direction);

  // Attempt to swap or move the window into a sibling container.
  if has_matching_tiling_direction {
    if let Some(sibling) =
      tiling_sibling_in_direction(&window_to_move, direction)
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
      &window_to_move.into(),
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
    None => invert_workspace_tiling_direction(
      window_to_move,
      direction,
      state,
      config,
    ),
    // Otherwise, move the container into the given ancestor. This could
    // simply be the container's direct parent.
    Some(target_ancestor) => insert_into_ancestor(
      &window_to_move,
      &target_ancestor,
      direction,
      state,
    ),
  }
}

/// Gets the next sibling `TilingWindow` or `SplitContainer` in the given
/// direction.
fn tiling_sibling_in_direction(
  window: &TilingWindow,
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
        &window_to_move.clone().into(),
        &parent,
        sibling_window.index(),
        state,
      )?;

      state
        .pending_sync
        .add_container_to_redraw(sibling_window)
        .add_container_to_redraw(window_to_move);
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
          &window_to_move.into(),
          &target_parent.clone().into(),
          target_index,
          state,
        )?;

        // TODO: Only redraw tiling containers.
        state
          .pending_sync
          .add_container_to_redraw(target_parent)
          .add_container_to_redraw(parent);
      }
    }
  };

  Ok(())
}

fn move_to_workspace_in_direction(
  window_to_move: &WindowContainer,
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

    if let WindowContainer::NonTilingWindow(window_to_move) =
      &window_to_move
    {
      window_to_move.set_insertion_target(None);
    }

    let target_index = match direction {
      Direction::Down | Direction::Right => 0,
      _ => workspace.child_count(),
    };

    move_container_within_tree(
      &window_to_move.clone().into(),
      &workspace.clone().into(),
      target_index,
      state,
    )?;

    state
      .pending_sync
      .add_containers_to_redraw(workspace.tiling_children())
      .add_containers_to_redraw(parent.tiling_children())
      .mark_cursor_jump();
  };

  Ok(())
}

fn invert_workspace_tiling_direction(
  window_to_move: TilingWindow,
  direction: &Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let workspace = window_to_move.workspace().context("No workspace.")?;

  // Get top-level tiling children of the workspace.
  let workspace_children = workspace
    .tiling_children()
    .filter(|container| container.id() != window_to_move.id())
    .collect::<Vec<_>>();

  // Create a new split container to wrap the window's siblings. For
  // example, in the layout H[1 V[2 3]] where container 3 is moved down,
  // we create a split container around 1 and 2. This results in
  // H[H[1 V[2 3]]], and V[H[1 V[2]] 3] after the tiling direction change.
  if workspace_children.len() > 1 {
    let split_container = SplitContainer::new(
      workspace.tiling_direction(),
      config.value.gaps.clone(),
    );

    wrap_in_split_container(
      &split_container,
      &workspace.clone().into(),
      &workspace_children,
    )?;
  }

  // Invert the tiling direction of the workspace.
  workspace.set_tiling_direction(workspace.tiling_direction().inverse());

  let target_index = match direction {
    Direction::Left | Direction::Up => 0,
    _ => workspace.child_count(),
  };

  // Depending on the direction, place the window either before or after
  // the split container.
  move_container_within_tree(
    &window_to_move.clone().into(),
    &workspace.clone().into(),
    target_index,
    state,
  )?;

  // Workspace might have redundant split containers after the tiling
  // direction change. For example, V[H[1 2] 3] where container 3 is moved
  // up results in H[3 H[1 2]], and needs to be flattened to H[3 1 2].
  flatten_child_split_containers(&workspace.clone().into())?;

  // Resize the window such that the split container and window are each
  // 0.5.
  resize_tiling_container(&window_to_move.into(), 0.5);

  state
    .pending_sync
    .add_containers_to_redraw(workspace.tiling_children());

  Ok(())
}

fn insert_into_ancestor(
  window_to_move: &TilingWindow,
  target_ancestor: &DirectionContainer,
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
        .is_some_and(|parent| parent == target_ancestor.clone().into())
    })
    .context("Window ancestor not found.")?;

  let target_index = match direction {
    Direction::Up | Direction::Left => window_ancestor.index(),
    _ => window_ancestor.index() + 1,
  };

  // Move the window into the container above.
  move_container_within_tree(
    &window_to_move.clone().into(),
    &target_ancestor.clone().into(),
    target_index,
    state,
  )?;

  state
    .pending_sync
    .add_containers_to_redraw(target_ancestor.tiling_children());

  Ok(())
}

fn move_floating_window(
  window_to_move: NonTilingWindow,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let new_position =
    new_floating_position(&window_to_move, direction, state)?;

  if let Some((position_rect, target_monitor)) = new_position {
    let monitor = window_to_move.monitor().context("No monitor.")?;

    // Mark window as needing DPI adjustment if it crosses monitors. The
    // handler for `PlatformEvent::LocationChanged` will update the
    // window's workspace if it goes out of bounds of its current
    // workspace.
    if monitor.id() != target_monitor.id()
      && monitor.has_dpi_difference(&target_monitor.into())?
    {
      window_to_move.set_has_pending_dpi_adjustment(true);
    }

    window_to_move.set_floating_placement(position_rect);
    state.pending_sync.add_container_to_redraw(window_to_move);
  }

  Ok(())
}

/// Returns a tuple of the new floating position and the target monitor.
fn new_floating_position(
  window_to_move: &NonTilingWindow,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<Option<(Rect, Monitor)>> {
  let monitor = window_to_move.monitor().context("No monitor.")?;
  let monitor_rect = monitor.native().working_rect()?.clone();
  let window_pos = window_to_move.native().frame_position()?;

  let is_on_monitor_edge = match direction {
    Direction::Up => window_pos.top == monitor_rect.top,
    Direction::Down => window_pos.bottom == monitor_rect.bottom,
    Direction::Left => window_pos.left == monitor_rect.left,
    Direction::Right => window_pos.right == monitor_rect.right,
  };

  // Window is on the edge of the monitor and should be moved to a
  // different monitor in the given direction.
  if is_on_monitor_edge {
    let next_monitor = state.monitor_in_direction(&monitor, direction)?;

    if let Some(next_monitor) = next_monitor {
      let monitor_rect = next_monitor.native().working_rect()?.clone();

      let position = snap_to_monitor_edge(
        &window_pos,
        &monitor_rect,
        &direction.inverse(),
      )
      .clamp(&monitor_rect);

      return Ok(Some((position, next_monitor)));
    }

    return Ok(None);
  }

  let (monitor_length, window_length) = match direction {
    Direction::Up | Direction::Down => {
      (monitor_rect.height(), window_pos.height())
    }
    _ => (monitor_rect.width(), window_pos.width()),
  };

  let length_delta = monitor_length - window_length;

  // Calculate the distance the window should move based on the ratio of
  // the window's length to the monitor's length.
  #[allow(clippy::cast_precision_loss)]
  let move_distance = match window_length as f32 / monitor_length as f32 {
    x if (0.0..0.2).contains(&x) => length_delta / 5,
    x if (0.2..0.4).contains(&x) => length_delta / 4,
    x if (0.4..0.6).contains(&x) => length_delta / 3,
    _ => length_delta / 2,
  };

  // Snap the window to the current monitor's edge if it's within 15px of
  // it after the move.
  let should_snap_to_edge = match direction {
    Direction::Up => {
      window_pos.top - move_distance - SNAP_DISTANCE < monitor_rect.top
    }
    Direction::Down => {
      window_pos.bottom + move_distance + SNAP_DISTANCE
        > monitor_rect.bottom
    }
    Direction::Left => {
      window_pos.left - move_distance - SNAP_DISTANCE < monitor_rect.left
    }
    Direction::Right => {
      window_pos.right + move_distance + SNAP_DISTANCE > monitor_rect.right
    }
  };

  if should_snap_to_edge {
    let position =
      snap_to_monitor_edge(&window_pos, &monitor_rect, direction);

    return Ok(Some((position, monitor)));
  }

  // Snap the window to the current monitor's inverse edge if it's in
  // between two monitors or outside the bounds of the current monitor.
  let should_snap_to_inverse_edge = match direction {
    Direction::Up => window_pos.bottom > monitor_rect.bottom,
    Direction::Down => window_pos.top < monitor_rect.top,
    Direction::Left => window_pos.right > monitor_rect.right,
    Direction::Right => window_pos.left < monitor_rect.left,
  };

  let position = if should_snap_to_inverse_edge {
    snap_to_monitor_edge(&window_pos, &monitor_rect, &direction.inverse())
  } else {
    window_pos.translate_in_direction(direction, move_distance)
  };

  Ok(Some((position, monitor)))
}

fn snap_to_monitor_edge(
  window_pos: &Rect,
  monitor_rect: &Rect,
  edge: &Direction,
) -> Rect {
  let (x, y) = match edge {
    Direction::Up => (window_pos.x(), monitor_rect.top),
    Direction::Down => {
      (window_pos.x(), monitor_rect.bottom - window_pos.height())
    }
    Direction::Left => (monitor_rect.left, window_pos.y()),
    Direction::Right => {
      (monitor_rect.right - window_pos.width(), window_pos.y())
    }
  };

  window_pos.translate_to_coordinates(x, y)
}
