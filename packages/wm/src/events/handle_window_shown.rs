use tracing::info;
use wm_common::{DisplayState, HideMethod, WindowState};
use wm_platform::NativeWindow;

use crate::{
  commands::window::manage_window,
  models::{NativeWindowProperties, WindowContainer},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_window_shown(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  if let Some(window) = found_window {
    info!("Window shown: {window}");

    // Update display state if window is already managed.
    if config.value.general.hide_method != HideMethod::PlaceInCorner
      && window.display_state() == DisplayState::Showing
    {
      window.set_display_state(DisplayState::Shown);
    } else {
      state.pending_sync.queue_container_to_redraw(window);
    }
  } else if let Some(old_tab) = find_background_tab(&native_window, state)
  {
    swap_tab_window(old_tab, native_window, state)?;
  } else {
    manage_window(native_window, None, state, config)?;
  }

  Ok(())
}

/// Finds a managed window from the same process that has become a
/// background tab (i.e. is no longer a root window).
#[cfg(target_os = "macos")]
fn find_background_tab(
  new_window: &NativeWindow,
  state: &WmState,
) -> Option<WindowContainer> {
  use wm_platform::NativeWindowExtMacOs;

  let new_pid = new_window.process_id();

  state.windows().into_iter().find(|window| {
    let is_same_process = window.native().process_id() == new_pid;
    let is_background_tab =
      is_same_process && !window.native().is_root_window().unwrap_or(true);

    is_background_tab
  })
}

#[cfg(not(target_os = "macos"))]
fn find_background_tab(
  _new_window: &NativeWindow,
  _state: &WmState,
) -> Option<WindowContainer> {
  None
}

/// Replaces a background tab's native window reference with the newly
/// active tab, preserving the window's position in the layout tree.
fn swap_tab_window(
  old_tab: WindowContainer,
  new_native: NativeWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let new_properties = NativeWindowProperties::try_from(&new_native)?;

  info!(
    "Tab switch: replacing {} (handle {}) with new tab (handle {})",
    old_tab,
    old_tab.native().id().0,
    new_native.id().0,
  );

  old_tab.set_native(new_native);
  old_tab.update_native_properties(|props| {
    props.title = new_properties.title;
    props.frame = new_properties.frame;
    props.is_minimized = new_properties.is_minimized;
    props.is_maximized = new_properties.is_maximized;
    props.is_resizable = new_properties.is_resizable;
  });

  // Redraw the window at its existing position/size. For tiling
  // windows this reapplies the tiled frame; for floating windows this
  // reapplies the stored floating placement.
  match old_tab.state() {
    WindowState::Tiling => {
      if let Some(parent) = old_tab.parent() {
        state.pending_sync.queue_container_to_redraw(parent);
      }
    }
    _ => {
      state.pending_sync.queue_container_to_redraw(old_tab);
    }
  }

  state.pending_sync.queue_focus_change();

  Ok(())
}
