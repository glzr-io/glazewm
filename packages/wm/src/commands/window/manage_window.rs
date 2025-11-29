use anyhow::Context;
use tracing::info;
use wm_common::{
  try_warn, LengthValue, RectDelta, TilingDirection, WindowRuleEvent, WindowState, WmEvent,
};
use wm_platform::{NativeWindow, Platform}; // <--- Import Platform

use crate::{
  commands::{
    container::{
      attach_container, detach_container, set_focused_descendant,
      wrap_in_split_container,
    },
    window::run_window_rules,
  },
  models::{
    Container, Monitor, NonTilingWindow, SplitContainer, TilingContainer,
    TilingWindow, WindowContainer, Workspace,
  },
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters, TilingSizeGetters,
    WindowGetters,
  },
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

    // Normally, a `PlatformEvent::WindowFocused` event is what triggers
    // focus effects and workspace reordering to be applied. However, when
    // a window is first launched, this event can come before the
    // window is managed, and so we need to force an update here.
    state.pending_sync.queue_focused_effect_update();
    state.pending_sync.queue_workspace_to_reorder(
      window.workspace().context("No workspace.")?,
    );

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
  // --- CHANGE: Use mouse_position() to find the correct monitor ---
  let nearest_monitor = if let Ok(mouse_pos) = Platform::mouse_position() {
    state.monitor_at_point(&mouse_pos)
  } else {
    None
  }
  .or_else(|| state.nearest_monitor(&native_window))
  .context("No nearest monitor.")?;

  let nearest_workspace = nearest_monitor
    .displayed_workspace()
    .context("No nearest workspace.")?;

  let gaps_config = config.value.gaps.clone();
  
  // Note: We use the cursor-based monitor here to determine window state
  let window_state =
    window_state_to_create(&native_window, &nearest_monitor, config)?;
  let is_tiling = window_state == WindowState::Tiling;

  // --- CHANGE: Force target workspace to be the one under cursor ---
  let target_workspace = match &target_parent {
      Some(parent) => parent.workspace().context("No target workspace.")?,
      None => nearest_workspace.clone(),
  };

  // Determine insertion. 
  // If floating, we check if the standard insertion logic (which uses focus)
  // matches our target monitor. If not, we force it to the target monitor.
  let (target_parent, target_index) = if !is_tiling {
    match target_parent {
      Some(parent) => (parent, 0),
      None => {
        let (focused_target, focused_idx) = insertion_target(&window_state, state)?;
        // If focus is on the same workspace as the cursor, use the smart insertion logic.
        // Otherwise, just append to the cursor's workspace.
        if focused_target.workspace().map(|w| w.id()) == Some(target_workspace.id()) {
            (focused_target, focused_idx)
        } else {
            (target_workspace.clone().into(), target_workspace.child_count())
        }
      },
    }
  } else {
    // For spiral tiling, we start with the workspace root
    (target_workspace.clone().into(), 0)
  };

  let prefers_centered = config
    .value
    .window_behavior
    .state_defaults
    .floating
    .centered;

  // Calculate where window should be placed when floating is enabled.
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
      gaps_config.clone(),
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

  // Implement BSWM spiral tiling with cursor awareness
  let (final_target_parent, final_target_index) = if is_tiling {
    // We use the target_workspace (derived from cursor) to find the spiral split
    let tiling_children_count = target_workspace.tiling_children().count();

    if tiling_children_count == 0 {
      (target_workspace.clone().into(), 0)
    } else {
      if let Some((parent_to_split, child_to_wrap)) =
        find_last_tiling_window_to_split(&target_workspace.clone().into())
      {
        let split_direction = match child_to_wrap.to_rect() {
          Ok(rect) => {
            if rect.width() > rect.height() {
              TilingDirection::Horizontal 
            } else {
              TilingDirection::Vertical 
            }
          }
          Err(_) => {
            if let Ok(direction_parent) = parent_to_split.as_direction_container() {
              direction_parent.tiling_direction().inverse()
            } else {
              TilingDirection::Horizontal
            }
          }
        };

        let split_container = SplitContainer::new(split_direction, gaps_config);

        wrap_in_split_container(
          &split_container,
          &parent_to_split,
          &[child_to_wrap],
        )?;

        (split_container.into(), 1)
      } else {
        (target_workspace.clone().into(), target_workspace.child_count())
      }
    }
  } else {
    (target_parent, target_index)
  };

  attach_container(
    &window_container.clone().into(),
    &final_target_parent,
    Some(final_target_index),
  )?;

  if nearest_monitor
    .has_dpi_difference(&window_container.clone().into())?
  {
    window_container.set_has_pending_dpi_adjustment(true);
  }

  Ok(window_container)
}

pub fn find_last_tiling_window_to_split(
  container: &Container,
) -> Option<(Container, TilingContainer)> {
  if let Ok(direction_container) = container.as_direction_container() {
    let tiling_children: Vec<TilingContainer> =
      direction_container.tiling_children().collect();

    if !tiling_children.is_empty() {
      let last_child = tiling_children[tiling_children.len() - 1].clone();

      if let Some(split) = last_child.as_split() {
        if let Some(result) =
          find_last_tiling_window_to_split(&split.clone().into())
        {
          return Some(result);
        }
      }

      return Some((container.clone(), last_child));
    }
  }
  None
}

pub fn rebuild_spiral_layout(
  workspace: &Workspace,
  windows: &[TilingWindow],
) -> anyhow::Result<()> {
  for win in windows {
    let _ = detach_container(win.clone().into());
  }

  let children_to_remove: Vec<Container> = workspace
    .children()
    .into_iter()
    .filter(|c| c.is_split())
    .collect();

  for child in children_to_remove {
    let _ = detach_container(child);
  }

  if let Some(first) = windows.first() {
    attach_container(
      &first.clone().into(),
      &workspace.clone().into(),
      None,
    )?;
  }

  for win in windows.iter().skip(1) {
    if let Some((parent_to_split, child_to_wrap)) =
      find_last_tiling_window_to_split(&workspace.clone().into())
    {
      let split_direction = match child_to_wrap.to_rect() {
        Ok(rect) => {
          if rect.width() > rect.height() {
            TilingDirection::Horizontal
          } else {
            TilingDirection::Vertical
          }
        }
        Err(_) => {
          if let Ok(direction_parent) = parent_to_split.as_direction_container() {
            direction_parent.tiling_direction().inverse()
          } else {
            TilingDirection::Horizontal
          }
        }
      };

      let gaps_config = (*win.gaps_config()).clone();
      let split_container = SplitContainer::new(split_direction, gaps_config);

      wrap_in_split_container(
        &split_container,
        &parent_to_split,
        &[child_to_wrap],
      )?;

      attach_container(
        &win.clone().into(),
        &split_container.into(),
        Some(1),
      )?;
    } else {
      attach_container(
        &win.clone().into(),
        &workspace.clone().into(),
        None,
      )?;
    }
  }

  Ok(())
}

fn window_state_to_create(
  native_window: &NativeWindow,
  nearest_monitor: &Monitor,
  config: &UserConfig,
) -> anyhow::Result<WindowState> {
  if native_window.is_minimized()? {
    return Ok(WindowState::Minimized);
  }

  let nearest_workspace = nearest_monitor
    .displayed_workspace()
    .context("No Workspace.")?;

  let monitor_rect = if config
    .outer_gaps_for_workspace(&nearest_workspace)
    .is_significant()
  {
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

  if !native_window.is_resizable() {
    return Ok(WindowState::Floating(
      config.value.window_behavior.state_defaults.floating.clone(),
    ));
  }

  Ok(WindowState::default_from_config(&config.value))
}

fn insertion_target(
  window_state: &WindowState,
  state: &WmState,
) -> anyhow::Result<(Container, usize)> {
  let focused_container =
    state.focused_container().context("No focused container.")?;

  let focused_workspace =
    focused_container.workspace().context("No workspace.")?;

  if *window_state == WindowState::Tiling {
    let sibling = match focused_container {
      Container::TilingWindow(_) => Some(focused_container),
      _ => focused_workspace
        .descendant_focus_order()
        .find(Container::is_tiling_window),
    };

    if let Some(sibling) = sibling {
      return match sibling.parent() {
        Some(parent) => Ok((parent, sibling.index() + 1)),
        None => {
          Ok((
            focused_workspace.clone().into(),
            focused_workspace.child_count(),
          ))
        }
      };
    }
  }

  Ok((
    focused_workspace.clone().into(),
    focused_workspace.child_count(),
  ))
}