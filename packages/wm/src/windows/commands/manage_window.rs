use anyhow::Context;
use tracing::info;

use crate::{
  common::{platform::NativeWindow, LengthValue, RectDelta},
  containers::{
    commands::{attach_container, set_focused_descendant},
    traits::{CommonGetters, PositionGetters},
    Container, WindowContainer,
  },
  monitors::Monitor,
  user_config::{UserConfig, WindowRuleEvent},
  windows::{
    commands::run_window_rules, traits::WindowGetters, NonTilingWindow,
    TilingWindow, WindowState,
  },
  wm_event::WmEvent,
  wm_state::WmState,
};

pub fn manage_window(
  native_window: NativeWindow,
  target_parent: Option<Container>,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  // Create the window instance.
  let window = create_window(native_window, target_parent, state, config)?;

  // Set the newly added window as focus descendant. This means the window
  // rules will be run as if the window is focused.
  set_focused_descendant(window.clone().into(), None);

  // Window might be detached if `ignore` command has been invoked.
  let updated_window = run_window_rules(
    window.clone(),
    WindowRuleEvent::Manage,
    state,
    config,
  )?;

  if let Some(window) = updated_window {
    // TODO: Log window details.
    info!("New window managed");

    state.emit_event(WmEvent::WindowManaged {
      managed_window: window.to_dto()?,
    });

    // OS focus should be set to the newly added window in case it's not
    // already focused.
    state.pending_sync.focus_change = true;

    // Sibling containers need to be redrawn if the window is tiling.
    state.pending_sync.containers_to_redraw.push(
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
  // Attach the new window as the first child of the target parent (if
  // provided), otherwise, add as a sibling of the focused container.
  let (target_parent, target_index) = match target_parent {
    Some(parent) => (parent, 0),
    None => insertion_target(state)?,
  };

  let target_workspace =
    target_parent.workspace().context("No target workspace.")?;

  let nearest_monitor = state
    .nearest_monitor(&native_window)
    .context("No nearest monitor.")?;

  let nearest_workspace = nearest_monitor
    .displayed_workspace()
    .context("No nearest workspace.")?;

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
  let floating_placement = match !is_same_workspace || prefers_centered {
    true => native_window
      .frame_position()?
      .translate_to_center(&target_workspace.to_rect()?),
    false => native_window.frame_position()?,
  }
  // Clamp the window size to 90% of the workspace size.
  .clamp_size(
    (target_workspace.to_rect()?.width() as f32 * 0.9) as i32,
    (target_workspace.to_rect()?.height() as f32 * 0.9) as i32,
  );

  // Window has no border delta unless it's later changed via the
  // `adjust_borders` command.
  let border_delta = RectDelta::new(
    LengthValue::from_px(0),
    LengthValue::from_px(0),
    LengthValue::from_px(0),
    LengthValue::from_px(0),
  );

  let inner_gap = config.value.gaps.inner_gap.clone();
  let window_state =
    window_state_to_create(&native_window, &nearest_monitor, config)?;

  let window_container: WindowContainer = match window_state {
    WindowState::Tiling => TilingWindow::new(
      None,
      native_window,
      None,
      border_delta,
      floating_placement,
      inner_gap,
      Vec::new(),
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
      Vec::new(),
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

  if native_window.is_fullscreen(&nearest_monitor.to_rect()?)? {
    return Ok(WindowState::Fullscreen(
      config
        .value
        .window_behavior
        .state_defaults
        .fullscreen
        .clone(),
    ));
  }

  // Initialize windows that can't be resized as floating.
  if !native_window.is_resizable() {
    return Ok(WindowState::Floating(
      config.value.window_behavior.state_defaults.floating.clone(),
    ));
  }

  Ok(WindowState::default_from_config(config))
}

fn insertion_target(
  state: &WmState,
) -> anyhow::Result<(Container, usize)> {
  let focused_container =
    state.focused_container().context("No focused container.")?;

  match focused_container.is_workspace() {
    true => Ok((focused_container, 0)),
    false => Ok((
      focused_container.parent().context("No insertion target.")?,
      focused_container.index() + 1,
    )),
  }
}
