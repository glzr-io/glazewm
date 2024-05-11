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
  wm_event::WmEvent,
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
      _ => (),
    }
  }

  Ok(())
}

fn update_tiling_window(
  window: TilingWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // Snap window to its original position if it's not being resized.
  if window.parent().context("No parent.")?.is_workspace()
    && window.tiling_siblings().count() == 0
  {
    state.containers_to_redraw.push(window.clone().into());
    return Ok(());
  }

  let monitor = window.monitor().context("Window has no monitor.")?;

  // Remove invisible borders from current placement to be able to compare
  // window width/height.
  let new_placement = window.native().placement();
  let adjusted_placement = Rect::from_ltrb(
    new_placement.left
      + window.border_delta().left.to_pixels(monitor.width()?),
    new_placement.top
      + window.border_delta().top.to_pixels(monitor.width()?),
    new_placement.right
      - window.border_delta().right.to_pixels(monitor.width()?),
    new_placement.bottom
      - window.border_delta().bottom.to_pixels(monitor.width()?),
  );

  let delta_width = adjusted_placement.width() - window.width()?;
  let delta_height = adjusted_placement.height() - window.height()?;

  resize_window(
    window.clone().into(),
    ResizeDimension::Width,
    LengthValue::new_px(delta_width as f32),
    state,
  )?;

  resize_window(
    window.clone().into(),
    ResizeDimension::Height,
    LengthValue::new_px(delta_height as f32),
    state,
  )?;

  Ok(())
}

fn update_floating_window(
  window: NonTilingWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // Update state with new location of the floating window.
  let new_placement = window.native().placement();
  window.set_floating_placement(new_placement);

  // Change floating window's parent workspace if moved out of its bounds.
  update_parent_workspace(window.clone().into(), state)?;
  Ok(())
}

fn update_parent_workspace(
  window: NonTilingWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // Get workspace that encompasses most of the window.
  let target_workspace = state
    .nearest_monitor(&window.native())
    .context("Couldn't find nearest monitor.")?
    .displayed_workspace()
    .context("Monitor has no displayed workspace.")?;

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
  )?;

  if window.is_focused() {
    state.emit_event(WmEvent::FocusedContainerMoved {
      focused_container: window.into(),
    });
  }

  Ok(())
}
