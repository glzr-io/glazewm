use anyhow::Context;
use wm_common::{
  try_warn, FullscreenStateConfig, TilingDirection, TilingLayout,
  WindowState,
};
use wm_platform::{LengthValue, Point, Rect};

use crate::{
  commands::{
    container::{move_container_within_tree, wrap_in_split_container},
    window::{set_window_size, update_window_state},
  },
  events::update_floating_window_position,
  models::{
    DirectionContainer, NonTilingWindow, SplitContainer, TilingContainer,
    WindowContainer,
  },
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters,
    TilingLayoutGetters, WindowGetters,
  },
  user_config::UserConfig,
  wm_state::WmState,
};

/// Handles the event for when a window is finished being moved or resized
/// by the user (e.g. via the window's drag handles).
///
/// This resizes the window if it's a tiling window and attach a dragged
/// floating window.
///
/// TODO: Move this to a better location - maybe a new `active_drag_ext`
/// mod.
pub fn handle_window_moved_or_resized_end(
  window: &WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(active_drag) = window.active_drag() else {
    return Ok(());
  };

  match &window {
    WindowContainer::NonTilingWindow(window) => {
      let is_maximized = try_warn!(window.native().is_maximized());

      window.update_native_properties(|properties| {
        properties.is_maximized = is_maximized;
      });

      let nearest_monitor = state
        .nearest_monitor(&window.native())
        .context("Failed to get workspace of nearest monitor.")?;

      let should_fullscreen = window.should_fullscreen(
        &nearest_monitor
          .displayed_workspace()
          .context("No workspace.")?,
      )?;

      if is_maximized || should_fullscreen {
        let fullscreen_state = if let WindowState::Fullscreen(
          fullscreen_state,
        ) = window.state()
        {
          fullscreen_state
        } else {
          config
            .value
            .window_behavior
            .state_defaults
            .fullscreen
            .clone()
        };

        let window = update_window_state(
          window.clone().into(),
          WindowState::Fullscreen(FullscreenStateConfig {
            maximized: is_maximized,
            ..fullscreen_state
          }),
          state,
          config,
        )?;

        window.set_active_drag(None);

        if is_maximized {
          // Dequeue the window from redraw if it's maximized, since the
          // window is already in the correct state.
          state
            .pending_sync
            .dequeue_container_from_redraw(window.clone());
        } else {
          // Force a redraw to snap the window to the monitor edges.
          // TODO: Skip redraw if it's already matches fullscreen frame.
          state.pending_sync.queue_container_to_redraw(window.clone());
        }

        return Ok(());
      }

      if active_drag.is_from_floating {
        update_floating_window_position(
          window,
          window.native_properties().frame,
          &nearest_monitor,
          state,
        )?;
        window.set_active_drag(None);
      } else {
        // Window is a temporary floating window that should be
        // reverted back to tiling.
        let window = drop_as_tiling_window(window, state, config)?;
        window.set_active_drag(None);
      }
    }
    WindowContainer::TilingWindow(window) => {
      tracing::info!(
        "Tiling window move/resize ended: {}",
        window.as_window_container()?
      );

      let frame = window.native_properties().frame;

      // Update the window's size based on the new frame position. This
      // means we use the actual window dimensions as the source of truth.
      set_window_size(
        window.clone().into(),
        Some(LengthValue::from_px(frame.width())),
        Some(LengthValue::from_px(frame.height())),
        state,
      )?;

      window.set_active_drag(None);

      // Force a redraw of the window to snap it back to its original
      // position. This is necessary when:
      // - The window is the only tiling window in the workspace.
      // - The window is not past the movement threshold for transitioning
      //   to floating while being dragged.
      // - Resizing in a direction that doesn't change the window's tiling
      //   size.
      state.pending_sync.queue_container_to_redraw(window.clone());
    }
  }

  Ok(())
}

/// Handles transition from temporary floating window to tiling window on
/// drag end.
#[allow(clippy::too_many_lines)]
fn drop_as_tiling_window(
  moved_window: &NonTilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<WindowContainer> {
  tracing::info!(
    "Tiling window drag ended: {}",
    moved_window.as_window_container()?
  );

  let mouse_pos = state.dispatcher.cursor_position()?;
  let mouse_workspace = state
    .monitor_at_point(&mouse_pos)
    .and_then(|monitor| monitor.displayed_workspace())
    .or_else(|| moved_window.workspace())
    .context("Couldn't find workspace for window drop.")?;

  // Get the workspace, split containers, and other windows under the
  // dragged window.
  let containers_at_pos = state
    .containers_at_point(&mouse_workspace.clone().into(), &mouse_pos)
    .into_iter()
    .filter(|container| container.id() != moved_window.id());

  // Get the deepest direction container under the dragged window.
  let target_parent: DirectionContainer = containers_at_pos
    .filter_map(|container| container.as_direction_container().ok())
    .filter(|container| {
      is_frontmost_direction_container_at_point(container, &mouse_pos)
    })
    .fold(mouse_workspace.into(), |acc, container| {
      let container_depth = container.ancestors().count();
      let acc_depth = acc.ancestors().count();

      if container_depth > acc_depth
        || (container_depth == acc_depth
          && focus_path(&container) < focus_path(&acc))
      {
        container
      } else {
        acc
      }
    });

  // If the target parent has no children (i.e. an empty workspace), then
  // add the window directly.
  if target_parent.tiling_children().count() == 0 {
    move_container_within_tree(
      &moved_window.clone().into(),
      &target_parent.clone().into(),
      0,
      state,
    )?;

    moved_window.set_insertion_target(None);

    return update_window_state(
      moved_window.as_window_container()?,
      WindowState::Tiling,
      state,
      config,
    );
  }

  let nearest_container =
    if target_parent.tiling_layout() == TilingLayout::Accordion {
      accordion_child_at_point(&target_parent, &mouse_pos)?
    } else {
      nearest_tiling_child(&target_parent, &mouse_pos)?
    }
    .context("No nearest container.")?;

  let tiling_direction = target_parent.tiling_direction();
  let drop_position =
    drop_position(&mouse_pos, &nearest_container.to_rect()?);

  let moved_window = update_window_state(
    moved_window.clone().into(),
    WindowState::Tiling,
    state,
    config,
  )?;

  let should_split = nearest_container.is_tiling_window()
    && match tiling_direction {
      TilingDirection::Horizontal => {
        drop_position == DropPosition::Top
          || drop_position == DropPosition::Bottom
      }
      TilingDirection::Vertical => {
        drop_position == DropPosition::Left
          || drop_position == DropPosition::Right
      }
    };

  if should_split {
    let split_container = SplitContainer::new(
      tiling_direction.inverse(),
      config.value.gaps.clone(),
    );

    wrap_in_split_container(
      &split_container,
      &target_parent.clone().into(),
      &[nearest_container],
    )?;

    let target_index = match drop_position {
      DropPosition::Top | DropPosition::Left => 0,
      _ => 1,
    };

    move_container_within_tree(
      &moved_window.clone().into(),
      &split_container.into(),
      target_index,
      state,
    )?;
  } else {
    let target_index = match drop_position {
      DropPosition::Top | DropPosition::Left => nearest_container.index(),
      _ => nearest_container.index() + 1,
    };

    move_container_within_tree(
      &moved_window.clone().into(),
      &target_parent.clone().into(),
      target_index,
      state,
    )?;
  }

  state.pending_sync.queue_container_to_redraw(target_parent);

  Ok(moved_window)
}

/// Gets a direction container's path through ancestor focus orders.
fn focus_path(container: &DirectionContainer) -> Vec<usize> {
  let mut path = container
    .self_and_ancestors()
    .map(|ancestor| ancestor.focus_index())
    .collect::<Vec<_>>();
  path.reverse();
  path
}

/// Whether a direction container is in the frontmost accordion subtree
/// at a point.
fn is_frontmost_direction_container_at_point(
  container: &DirectionContainer,
  point: &Point,
) -> bool {
  for accordion_ancestor in container
    .ancestors()
    .filter_map(|ancestor| ancestor.as_direction_container().ok())
    .filter(|ancestor| ancestor.tiling_layout() == TilingLayout::Accordion)
  {
    let Some(topmost_child) =
      topmost_accordion_child_at_point(&accordion_ancestor, point)
        .ok()
        .flatten()
    else {
      continue;
    };

    let direct_child = container.self_and_ancestors().find(|candidate| {
      candidate
        .parent()
        .is_some_and(|parent| parent.id() == accordion_ancestor.id())
    });

    if direct_child.is_some_and(|child| child.id() != topmost_child.id()) {
      return false;
    }
  }

  true
}

/// Gets the direct accordion child containing the topmost leaf window at a
/// point.
fn accordion_child_at_point(
  parent: &DirectionContainer,
  point: &Point,
) -> anyhow::Result<Option<TilingContainer>> {
  Ok(
    topmost_accordion_child_at_point(parent, point)?
      .or_else(|| parent.tiling_children().next()),
  )
}

/// Gets the direct accordion child containing the topmost leaf window at a
/// point, without falling back when the point is only over empty space.
fn topmost_accordion_child_at_point(
  parent: &DirectionContainer,
  point: &Point,
) -> anyhow::Result<Option<TilingContainer>> {
  for leaf in parent.descendant_focus_order() {
    if !leaf.is_tiling_window() {
      continue;
    }

    let leaf = leaf.as_tiling_container()?;

    if leaf.to_rect()?.contains_point(point) {
      return Ok(leaf.self_and_ancestors().find_map(|candidate| {
        candidate
          .parent()
          .is_some_and(|candidate_parent| {
            candidate_parent.id() == parent.id()
          })
          .then(|| candidate.as_tiling_container().ok())
          .flatten()
      }));
    }
  }

  Ok(None)
}

/// Gets the tiling child nearest to a point in a tiles layout.
fn nearest_tiling_child(
  parent: &DirectionContainer,
  point: &Point,
) -> anyhow::Result<Option<TilingContainer>> {
  parent.tiling_children().try_fold(
    None,
    |acc: Option<TilingContainer>, container| match acc {
      Some(acc) => {
        let is_nearer = acc.to_rect()?.distance_to_point(point)
          < container.to_rect()?.distance_to_point(point);

        anyhow::Ok(Some(if is_nearer { acc } else { container }))
      }
      None => Ok(Some(container)),
    },
  )
}

/// Represents where the window was dropped over another.
#[derive(Debug, Clone, PartialEq)]
enum DropPosition {
  Top,
  Bottom,
  Left,
  Right,
}

/// Gets the drop position for a window based on the mouse position.
///
/// This approach divides the window rect into an "X", creating four
/// triangular quadrants, to determine which side the cursor is closest to.
fn drop_position(mouse_pos: &Point, rect: &Rect) -> DropPosition {
  let delta_x = mouse_pos.x - rect.center_point().x;
  let delta_y = mouse_pos.y - rect.center_point().y;

  if delta_x.abs() > delta_y.abs() {
    // Window is in the left or right triangle.
    if delta_x > 0 {
      DropPosition::Right
    } else {
      DropPosition::Left
    }
  } else {
    // Window is in the top or bottom triangle.
    if delta_y > 0 {
      DropPosition::Bottom
    } else {
      DropPosition::Top
    }
  }
}

#[cfg(test)]
mod tests {
  use std::collections::VecDeque;

  use wm_common::{GapsConfig, TilingDirection, TilingLayout};
  use wm_platform::{LengthValue, Point};

  use super::{
    accordion_child_at_point, is_frontmost_direction_container_at_point,
  };
  use crate::{
    models::{Monitor, SplitContainer, TilingWindow, Workspace},
    traits::{CommonGetters, PositionGetters, TilingLayoutGetters},
  };

  #[test]
  fn accordion_hit_testing_ignores_gaps_inside_focused_split() {
    let gaps_config = GapsConfig {
      inner_gap: LengthValue::from_px(20),
      ..GapsConfig::default()
    };
    let split_windows = [
      TilingWindow::mock().gaps_config(gaps_config.clone()).call(),
      TilingWindow::mock().gaps_config(gaps_config.clone()).call(),
    ];
    let focused_split = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .gaps_config(gaps_config.clone())
      .tiling_containers(
        split_windows.iter().cloned().map(Into::into).collect(),
      )
      .call();
    let underlying_window =
      TilingWindow::mock().gaps_config(gaps_config.clone()).call();
    let workspace = Workspace::mock()
      .tiling_direction(TilingDirection::Horizontal)
      .gaps_config(gaps_config)
      .tiling_containers(vec![
        focused_split.clone().into(),
        underlying_window.clone().into(),
      ])
      .call();

    workspace.set_tiling_layout(TilingLayout::Accordion);
    *workspace.borrow_child_focus_order_mut() =
      VecDeque::from([focused_split.id(), underlying_window.id()]);
    let _monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();

    let upper_rect = split_windows[0]
      .to_rect()
      .expect("Upper split window should have a rect.");
    let lower_rect = split_windows[1]
      .to_rect()
      .expect("Lower split window should have a rect.");
    let split_rect = focused_split
      .to_rect()
      .expect("Focused split should have a rect.");
    let underlying_rect = underlying_window
      .to_rect()
      .expect("Underlying window should have a rect.");
    let gap_point = Point {
      x: i32::midpoint(
        split_rect.left.max(underlying_rect.left),
        split_rect.right.min(underlying_rect.right),
      ),
      y: i32::midpoint(upper_rect.bottom, lower_rect.top),
    };
    let leaf_point = upper_rect.center_point();

    assert!(split_rect.contains_point(&gap_point));
    assert!(underlying_rect.contains_point(&gap_point));
    assert!(!upper_rect.contains_point(&gap_point));
    assert!(!lower_rect.contains_point(&gap_point));
    assert!(is_frontmost_direction_container_at_point(
      &focused_split.clone().into(),
      &leaf_point
    ));
    assert_eq!(
      accordion_child_at_point(&workspace.clone().into(), &leaf_point)
        .expect("Accordion leaf hit test should succeed."),
      Some(focused_split.clone().into())
    );
    assert!(!is_frontmost_direction_container_at_point(
      &focused_split.clone().into(),
      &gap_point
    ));
    assert_eq!(
      accordion_child_at_point(&workspace.into(), &gap_point)
        .expect("Accordion hit test should succeed."),
      Some(underlying_window.into())
    );
  }
}
