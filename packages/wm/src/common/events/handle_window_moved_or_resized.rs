use std::collections::VecDeque;

use anyhow::Context;
use tracing::{debug, info};
use windows::Win32::{Foundation, UI::WindowsAndMessaging::GetCursorPos};

use crate::{
  common::{
    platform::{MouseMoveEvent, NativeWindow, Platform},
    LengthValue, Point, Rect,
  },
  containers::{
    traits::{CommonGetters, PositionGetters},
    Container, WindowContainer,
  },
  user_config::UserConfig,
  windows::{
    commands::resize_window, traits::WindowGetters, NonTilingWindow,
    TilingWindow,
  },
  wm_state::WmState,
};

/// Handles the event for when a window is finished being moved or resized
/// by the user (e.g. via the window's drag handles).
///
/// This resizes the window if it's a tiling window.
pub fn handle_window_moved_or_resized(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  if let Some(WindowContainer::TilingWindow(window)) = found_window {
    // TODO: Log window details.

    let parent = window.parent().context("No parent.")?;

    // Snap window to its original position if it's the only window in the
    // workspace.
    if parent.is_workspace() && window.tiling_siblings().count() == 0 {
      state.pending_sync.containers_to_redraw.push(window.into());
      return Ok(());
    }

    let new_rect = window.native().refresh_frame_position()?;
    let old_rect = window.to_rect()?;

    let width_delta = new_rect.width() - old_rect.width();
    let height_delta = new_rect.height() - old_rect.height();

    let has_window_moved = match (width_delta, height_delta) {
      (0, 0) => true,
      _ => false,
    };

    return match has_window_moved {
      true => window_moved(window, state, config),
      false => window_resized(window, state, width_delta, height_delta),
    };
  }

  Ok(())
}

/// Handles window move events
fn window_moved(
  moved_window: TilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("Tiling window moved");

  let workspace =
    moved_window.workspace().context("Couldn't find a workspace")?;

  let mouse_position = Platform::mouse_position()?;

  let children_at_mouse_position: Vec<_> = workspace
    .descendants()
    .filter_map(|container| match container {
      Container::TilingWindow(tiling) => {
        Some(tiling)
      }
      _ => None,
    })
    .filter(|c| {
        let frame = c.to_rect();
        info!("{:?}", frame);
        info!("{:?}", mouse_position);
        frame.unwrap().contains_point(&mouse_position)
    })
      .filter(|window| window.id() != moved_window.id())
    .collect();

  if children_at_mouse_position.is_empty() {
    return Ok(())
  }

  let window_under_cursor = children_at_mouse_position.into_iter().next().unwrap();

  // let should_split = match window_under_cursor {
  //   Some(WindowContainer::TilingWindow(window)) => {
  //     match window.parent().context("Couldn't find parent")? {
  //       crate::containers::Container::Split(_) => ShouldSplit::No,
  //       _ => {
  //         todo!()
  //       }
  //     }
  //   }
  //   Some(WindowContainer::NonTilingWindow(window)) => {
  //     let rect = window
  //       .native()
  //       .frame_position()
  //       .context("couldn't get the window frame")?;
  //
  //     if rect.width() > rect.height() {
  //       ShouldSplit::Vertically
  //     } else {
  //       ShouldSplit::Horizontally
  //     }
  //   }
  //   None => return Ok(()),
  // };

  Ok(())
}

enum ShouldSplit {
  Vertically,
  Horizontally,
  No,
}

/// Handles window resize events
fn window_resized(
  window: TilingWindow,
  state: &mut WmState,
  width_delta: i32,
  height_delta: i32,
) -> anyhow::Result<()> {
  info!("Tiling window resized");
  resize_window(
    window.clone().into(),
    Some(LengthValue::from_px(width_delta)),
    Some(LengthValue::from_px(height_delta)),
    state,
  )
}
