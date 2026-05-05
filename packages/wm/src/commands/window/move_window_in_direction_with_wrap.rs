use anyhow::Context;
use wm_common::{TilingDirection, WindowState};
use wm_platform::{Direction, Rect};

use crate::{
  commands::{
    container::{
      flatten_child_split_containers, flatten_split_container,
      move_container_within_tree, resize_tiling_container,
      set_focused_descendant, wrap_in_split_container,
    },
    workspace::{activate_workspace, focus_workspace}, /* 新增 focus_workspace 的引入 */
  },
  models::{
    DirectionContainer, Monitor, NonTilingWindow, SplitContainer,
    TilingContainer, TilingWindow, WindowContainer, WorkspaceTarget,
  },
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters, WindowGetters,
  },
  user_config::UserConfig,
  wm_state::WmState,
};

const SNAP_DISTANCE: i32 = 15;

pub fn move_window_in_direction_with_wrap(
  window: WindowContainer,
  direction: &Direction,
  follow_focus: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  match window {
    WindowContainer::TilingWindow(window) => {
      move_tiling_window(window, direction, follow_focus, state, config)
    }
    WindowContainer::NonTilingWindow(non_tiling_window) => {
      match non_tiling_window.state() {
        WindowState::Floating(_) => move_floating_window(
          non_tiling_window,
          direction,
          follow_focus,
          state,
          config,
        ),
        WindowState::Fullscreen(_) => move_to_workspace_in_direction(
          &non_tiling_window.into(),
          direction,
          follow_focus,
          state,
          config,
        ),
        _ => Ok(()),
      }
    }
  }
}

fn move_tiling_window(
  window_to_move: TilingWindow,
  direction: &Direction,
  follow_focus: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
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

  if (has_matching_tiling_direction
    || window_to_move.tiling_siblings().count() == 0)
    && parent.is_workspace()
  {
    return move_to_workspace_in_direction(
      &window_to_move.into(),
      direction,
      follow_focus,
      state,
      config,
    );
  }

  let target_ancestor = parent.ancestors().find_map(|ancestor| {
    ancestor.as_direction_container().ok().filter(|ancestor| {
      ancestor.tiling_direction()
        == TilingDirection::from_direction(direction)
    })
  });

  match target_ancestor {
    None => invert_workspace_tiling_direction(
      window_to_move,
      direction,
      state,
      config,
    ),
    Some(target_ancestor) => insert_into_ancestor(
      &window_to_move,
      &target_ancestor,
      direction,
      state,
    ),
  }
}

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
      move_container_within_tree(
        &window_to_move.clone().into(),
        &parent,
        sibling_window.index(),
        state,
      )?;
      state
        .pending_sync
        .queue_container_to_redraw(sibling_window)
        .queue_container_to_redraw(window_to_move);
    }
    TilingContainer::Split(sibling_split) => {
      let sibling_descendant =
        sibling_split.descendant_in_direction(&direction.inverse());

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

        state
          .pending_sync
          .queue_container_to_redraw(target_parent)
          .queue_containers_to_redraw(parent.tiling_children());
      }
    }
  }

  Ok(())
}

fn move_to_workspace_in_direction(
  window_to_move: &WindowContainer,
  direction: &Direction,
  follow_focus: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let parent = window_to_move.parent().context("No parent.")?;
  let workspace = window_to_move.workspace().context("No workspace.")?;
  let monitor = parent.monitor().context("No monitor.")?;

  let mut target_workspace = state
    .monitor_in_direction(&monitor, direction)?
    .and_then(|m| m.displayed_workspace());

  if target_workspace.is_none() {
    let workspace_target = match direction {
      Direction::Left | Direction::Up => WorkspaceTarget::Previous,
      Direction::Right | Direction::Down => WorkspaceTarget::Next,
    };

    let (target_name, target_ws) =
      state.workspace_by_target(&workspace, workspace_target, config)?;

    target_workspace = match target_ws {
      Some(_) => target_ws,
      _ => match target_name {
        Some(name) => {
          activate_workspace(Some(&name), None, state, config)?;
          state.workspace_by_name(&name)
        }
        _ => None,
      },
    };
  }

  if let Some(target_workspace) = target_workspace {
    if monitor.has_dpi_difference(&target_workspace.clone().into())? {
      window_to_move.set_has_pending_dpi_adjustment(true);
    }

    window_to_move.set_floating_placement(
      window_to_move
        .floating_placement()
        .translate_to_center(&target_workspace.to_rect()?),
    );

    if let WindowContainer::NonTilingWindow(window_to_move) =
      &window_to_move
    {
      window_to_move.set_insertion_target(None);
    }

    let target_index = match direction {
      Direction::Down | Direction::Right => 0,
      _ => target_workspace.child_count(),
    };

    // 如果要求跟随焦点，则原工作空间不再重新分配焦点
    let focus_target = if follow_focus {
      None
    } else {
      state.focus_target_after_removal(window_to_move)
    };

    move_container_within_tree(
      &window_to_move.clone().into(),
      &target_workspace.clone().into(),
      target_index,
      state,
    )?;

    if follow_focus {
      // Activate the target workspace and force focus onto the moved
      // window.
      focus_workspace(
        WorkspaceTarget::Name(target_workspace.config().name.clone()),
        state,
        config,
      )?;
      set_focused_descendant(&window_to_move.clone().into(), None);
      state.pending_sync.queue_focus_change().queue_cursor_jump();
    } else if let Some(focus_target) = focus_target {
      // The focus remains on the other windows in the original workspace.
      set_focused_descendant(
        &focus_target,
        Some(&workspace.clone().into()),
      );
    }

    state
      .pending_sync
      .queue_container_to_redraw(window_to_move.clone())
      .queue_containers_to_redraw(target_workspace.tiling_children())
      .queue_containers_to_redraw(parent.tiling_children())
      .queue_cursor_jump()
      .queue_workspace_to_reorder(target_workspace);
  }

  Ok(())
}

fn invert_workspace_tiling_direction(
  window_to_move: TilingWindow,
  direction: &Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let workspace = window_to_move.workspace().context("No workspace.")?;
  let workspace_children = workspace
    .tiling_children()
    .filter(|container| container.id() != window_to_move.id())
    .collect::<Vec<_>>();

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

  workspace.set_tiling_direction(workspace.tiling_direction().inverse());

  let target_index = match direction {
    Direction::Left | Direction::Up => 0,
    _ => workspace.child_count(),
  };

  move_container_within_tree(
    &window_to_move.clone().into(),
    &workspace.clone().into(),
    target_index,
    state,
  )?;
  flatten_child_split_containers(&workspace.clone().into())?;
  resize_tiling_container(&window_to_move.into(), 0.5);

  state
    .pending_sync
    .queue_containers_to_redraw(workspace.tiling_children());
  Ok(())
}

fn insert_into_ancestor(
  window_to_move: &TilingWindow,
  target_ancestor: &DirectionContainer,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
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

  move_container_within_tree(
    &window_to_move.clone().into(),
    &target_ancestor.clone().into(),
    target_index,
    state,
  )?;
  state
    .pending_sync
    .queue_containers_to_redraw(target_ancestor.tiling_children());

  Ok(())
}

fn move_floating_window(
  window_to_move: NonTilingWindow,
  direction: &Direction,
  follow_focus: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let new_position =
    new_floating_position(&window_to_move, direction, state)?;

  if let Some((position_rect, target_monitor)) = new_position {
    let monitor = window_to_move.monitor().context("No monitor.")?;

    if monitor.id() != target_monitor.id()
      && monitor.has_dpi_difference(&target_monitor.clone().into())?
    {
      window_to_move.set_has_pending_dpi_adjustment(true);
    }

    window_to_move.set_floating_placement(position_rect);
    state.pending_sync.queue_container_to_redraw(window_to_move);
  } else {
    // Wrap Logic for Floating Windows
    let workspace_target = match direction {
      Direction::Left | Direction::Up => WorkspaceTarget::Previous,
      Direction::Right | Direction::Down => WorkspaceTarget::Next,
    };

    let workspace = window_to_move.workspace().context("No workspace.")?;
    let (target_name, target_ws) =
      state.workspace_by_target(&workspace, workspace_target, config)?;

    let target_workspace = match target_ws {
      Some(_) => target_ws,
      _ => match target_name {
        Some(name) => {
          activate_workspace(Some(&name), None, state, config)?;
          state.workspace_by_name(&name)
        }
        _ => None,
      },
    };

    if let Some(target_workspace) = target_workspace {
      let focus_target = if follow_focus {
        None
      } else {
        state.focus_target_after_removal(&window_to_move.clone().into())
      };

      move_container_within_tree(
        &window_to_move.clone().into(),
        &target_workspace.clone().into(),
        target_workspace.child_count(),
        state,
      )?;

      let monitor_rect = target_workspace
        .monitor()
        .context("No monitor.")?
        .native_properties()
        .working_area
        .clone();
      let window_pos = window_to_move.native_properties().frame;
      let position = snap_to_monitor_edge(
        &window_pos,
        &monitor_rect,
        &direction.inverse(),
      );

      window_to_move.set_floating_placement(position);

      if follow_focus {
        focus_workspace(
          WorkspaceTarget::Name(target_workspace.config().name.clone()),
          state,
          config,
        )?;
        set_focused_descendant(&window_to_move.clone().into(), None);
        state.pending_sync.queue_focus_change().queue_cursor_jump();
      } else if let Some(focus_target) = focus_target {
        set_focused_descendant(
          &focus_target,
          Some(&workspace.clone().into()),
        );
      }

      state
        .pending_sync
        .queue_container_to_redraw(window_to_move.clone())
        .queue_workspace_to_reorder(target_workspace);
    }
  }

  Ok(())
}

fn new_floating_position(
  window_to_move: &NonTilingWindow,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<Option<(Rect, Monitor)>> {
  let monitor = window_to_move.monitor().context("No monitor.")?;
  let monitor_rect = monitor.native_properties().working_area;
  let window_pos = window_to_move.native_properties().frame;

  let is_on_monitor_edge = match direction {
    Direction::Up => window_pos.top == monitor_rect.top,
    Direction::Down => window_pos.bottom == monitor_rect.bottom,
    Direction::Left => window_pos.left == monitor_rect.left,
    Direction::Right => window_pos.right == monitor_rect.right,
  };

  if is_on_monitor_edge {
    let next_monitor = state.monitor_in_direction(&monitor, direction)?;

    if let Some(next_monitor) = next_monitor {
      let monitor_rect = next_monitor.native().working_area()?.clone();
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

  #[allow(clippy::cast_precision_loss)]
  let move_distance = match window_length as f32 / monitor_length as f32 {
    x if (0.0..0.2).contains(&x) => length_delta / 5,
    x if (0.2..0.4).contains(&x) => length_delta / 4,
    x if (0.4..0.6).contains(&x) => length_delta / 3,
    _ => length_delta / 2,
  };

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
