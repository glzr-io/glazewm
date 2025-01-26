use anyhow::Context;
use tracing::info;
use wm_common::{
  try_warn, LengthValue, RectDelta, WindowRuleEvent, WindowState, WmEvent,
};
use wm_platform::NativeWindow;

use crate::{
  commands::{
    container::{attach_container, set_focused_descendant},
    window::run_window_rules,
  },
  models::{
    Container, Monitor, NonTilingWindow, TilingWindow, WindowContainer,
  },
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn manage_window(
  native_window: NativeWindow,
  target_parent: Option<Container>,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  // Create the window instance. This may fail if the window handle has
  // already been destroyed.
  let window =
    try_warn!(create_window(native_window, target_parent, state, config));

  // Set the newly added window as focus descendant. This means the window
  // rules will be run as if the window is focused.
  set_focused_descendant(&window.clone().into(), None);

  // Window might be detached if `ignore` command has been invoked.
  let updated_window = run_window_rules(
    window.clone(),
    &WindowRuleEvent::Manage,
    state,
    config,
  )?;

  if let Some(window) = updated_window {
    info!("New window managed: {window}");

    state.emit_event(WmEvent::WindowManaged {
      managed_window: window.to_dto()?,
    });

    // OS focus should be set to the newly added window in case it's not
    // already focused.
    state.pending_sync.queue_focus_change();

    // Sibling containers need to be redrawn if the window is tiling.
    state.pending_sync.queue_container_to_redraw(
      if window.state() == WindowState::Tiling {
        window.parent().context("No parent.")?
      } else {
        window.into()
      },
    );
  }

  Ok(())
}

fn create_window(
  native_window: NativeWindow,
  target_parent: Option<Container>,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<WindowContainer> {
  let nearest_monitor = state
    .nearest_monitor(&native_window)
    .context("No nearest monitor.")?;

  let nearest_workspace = nearest_monitor
    .displayed_workspace()
    .context("No nearest workspace.")?;

  let gaps_config = config.value.gaps.clone();
  let window_state =
    window_state_to_create(&native_window, &nearest_monitor, config)?;

  // Attach the new window as the first child of the target parent (if
  // provided), otherwise, add as a sibling of the focused container.
  let (target_parent, target_index) = match target_parent {
    Some(parent) => (parent, 0),
    None => insertion_target(&window_state, state)?,
  };

  let target_workspace =
    target_parent.workspace().context("No target workspace.")?;

  let prefers_centered = config
    .value
    .window_behavior
    .state_defaults
    .floating
    .centered;

  // Calculate where window should be placed when floating is enabled. Use
  // the original width/height of the window and optionally position it in
  // the center of the workspace.
  let is_same_workspace = nearest_workspace.id() == target_workspace.id();
  let floating_placement = {
    let placement = if !is_same_workspace || prefers_centered {
      native_window
        .frame_position()?
        .translate_to_center(&target_workspace.to_rect()?)
    } else {
      native_window.frame_position()?
    };

    // Clamp the window size to 90% of the workspace size.
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    placement.clamp_size(
      (target_workspace.to_rect()?.width() as f32 * 0.9) as i32,
      (target_workspace.to_rect()?.height() as f32 * 0.9) as i32,
    )
  };

  // Window has no border delta unless it's later changed via the
  // `adjust_borders` command.
  let border_delta = RectDelta::new(
    LengthValue::from_px(0),
    LengthValue::from_px(0),
    LengthValue::from_px(0),
    LengthValue::from_px(0),
  );

  let window_container: WindowContainer = match window_state {
    WindowState::Tiling => TilingWindow::new(
      None,
      native_window,
      None,
      border_delta,
      floating_placement,
      false,
      gaps_config,
      Vec::new(),
      None,
    )
    .into(),
    _ => NonTilingWindow::new(
      None,
      native_window,
      window_state,
      None,
      border_delta,
      None,
      floating_placement,
      false,
      Vec::new(),
      None,
    )
    .into(),
  };

  attach_container(
    &window_container.clone().into(),
    &target_parent,
    Some(target_index),
  )?;

  // The OS might spawn the window on a different monitor to the target
  // parent, so adjustments might need to be made because of DPI.
  if nearest_monitor
    .has_dpi_difference(&window_container.clone().into())?
  {
    window_container.set_has_pending_dpi_adjustment(true);
  }

  Ok(window_container)
}

/// Gets the initial state for a window based on its native state.
///
/// Note that maximized windows are initialized as tiling.
fn window_state_to_create(
  native_window: &NativeWindow,
  nearest_monitor: &Monitor,
  config: &UserConfig,
) -> anyhow::Result<WindowState> {
  if native_window.is_minimized()? {
    return Ok(WindowState::Minimized);
  }

  let monitor_rect = if config.has_outer_gaps() {
    nearest_monitor.native().working_rect()?.clone()
  } else {
    nearest_monitor.to_rect()?
  };

  if native_window.is_fullscreen(&monitor_rect)? {
    return Ok(WindowState::Fullscreen(
      config
        .value
        .window_behavior
        .state_defaults
        .fullscreen
        .clone(),
    ));
  }

  // Initialize windows that can't be resized and popup windows as floating.
  if !native_window.is_resizable() 
    || native_window.is_popup() {
    return Ok(WindowState::Floating(
      config.value.window_behavior.state_defaults.floating.clone(),
    ));
  }
    
  Ok(WindowState::default_from_config(&config.value))
}

/// Gets where to insert a new window in the container tree.
///
/// Rules:
/// - For non-tiling windows: Always append to the workspace.
/// - For tiling windows:
///   1. Try to insert after the focused tiling window if one exists.
///   2. If a non-tiling window is focused, try to insert after the first
///      tiling window found.
///   3. If no tiling windows exist, append to the workspace.
///
/// Returns tuple of (parent container, insertion index).
fn insertion_target(
  window_state: &WindowState,
  state: &WmState,
) -> anyhow::Result<(Container, usize)> {
  let focused_container =
    state.focused_container().context("No focused container.")?;

  let focused_workspace =
    focused_container.workspace().context("No workspace.")?;

  // For tiling windows, try to find a suitable tiling window to insert
  // next to.
  if *window_state == WindowState::Tiling {
    let sibling = match focused_container {
      Container::TilingWindow(_) => Some(focused_container),
      _ => focused_workspace
        .descendant_focus_order()
        .find(Container::is_tiling_window),
    };

    if let Some(sibling) = sibling {
      return Ok((
        sibling.parent().context("No parent.")?,
        sibling.index() + 1,
      ));
    }
  }

  // Default to appending to workspace.
  Ok((
    focused_workspace.clone().into(),
    focused_workspace.child_count(),
  ))
}
