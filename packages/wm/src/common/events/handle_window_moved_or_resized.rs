use anyhow::Context;
use tracing::info;

use crate::{
  common::{platform::NativeWindow, LengthValue, Rect, ResizeDimension},
  containers::{
    commands::move_container_within_tree,
    traits::{CommonGetters, PositionGetters},
    WindowContainer,
  },
  windows::{
    commands::resize_window, traits::WindowGetters, NonTilingWindow,
    TilingWindow, WindowState,
  },
  wm_state::WmState,
};

pub fn handle_window_moved_or_resized(
  native_window: NativeWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  if let Some(window) = found_window {
    // TODO: Log window details.
    info!("Window moved/resized");

    match window {
      WindowContainer::TilingWindow(tiling_window) => {
        update_tiling_window(tiling_window, state)?;
      }
      WindowContainer::NonTilingWindow(non_tiling_window) => {
        if matches!(non_tiling_window.state(), WindowState::Floating(_)) {
          update_floating_window(non_tiling_window, state)?;
        }
      }
    }
  }

  Ok(())
}

fn update_tiling_window(
  window: TilingWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let parent = window.parent().context("No parent.")?;

  // Snap window to its original position if it's the only window in the
  // workspace.
  if parent.is_workspace() && window.tiling_siblings().count() == 0 {
    state.containers_to_redraw.push(window.into());
    return Ok(());
  }

  let new_position = window.native().outer_position()?;
  let delta_width = new_position.width() - window.width()?;
  let delta_height = new_position.height() - window.height()?;

  resize_window(
    window.clone().into(),
    ResizeDimension::Width,
    LengthValue::new_px(delta_width as f32),
    state,
  )?;

  resize_window(
    window.into(),
    ResizeDimension::Height,
    LengthValue::new_px(delta_height as f32),
    state,
  )
}

fn update_floating_window(
  window: NonTilingWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // Update state with new location of the floating window.
  let new_position = window.native().outer_position()?;
  window.set_floating_placement(new_position);

  // Change floating window's parent workspace if moved out of its bounds.
  update_parent_workspace(window.clone().into(), state)
}

fn update_parent_workspace(
  window: NonTilingWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // Get workspace that encompasses most of the window.
  let target_workspace = state
    .nearest_monitor(&window.native())
    .and_then(|monitor| monitor.displayed_workspace())
    .context("Failed to get workspace of nearest monitor.")?;

  // Ignore if window is still within the bounds of its current workspace.
  if target_workspace.id()
    == window.workspace().context("No workspace.")?.id()
  {
    return Ok(());
  }

  // Change the window's parent workspace.
  move_container_within_tree(
    window.clone().into(),
    target_workspace.clone().into(),
    target_workspace.child_count(),
    state,
  )
}
