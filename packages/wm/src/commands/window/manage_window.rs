use std::sync::Mutex;

use anyhow::Context;
use tracing::{debug, info};
use tray_icon::menu::accelerator::Code::Insert;
use wm_common::{
  try_warn, LengthValue, RectDelta, TilingDirection, TilingLayout,
  WindowRuleEvent, WindowState, WmEvent,
};
use wm_platform::NativeWindow;

use crate::{
  commands::{
    container::{attach_container, set_focused_descendant},
    window::run_window_rules,
  },
  models::{
    Container, Monitor, NonTilingWindow, SplitContainer, TilingWindow,
    WindowContainer, Workspace,
  },
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters,
    TilingSizeGetters, WindowGetters,
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
  // TODO - learn why
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
  let (target_parent, target_index) =
    insertion_target(&target_parent, &window_state, state, config)?;

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

  // Initialize non-resizable windows and popups as floating.
  if !native_window.is_resizable() || native_window.is_popup() {
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
  target_parent: &Option<Container>,
  window_state: &WindowState,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<(Container, usize)> {
  // let target_workspace = if let Some(target) = target_parent {
  //   if target.is_workspace() {
  //     target.workspace().context("No workspace.")?
  //   } else {
  //     target.parent().context("No parent.")?.workspace().context("No
  // workspace.")?   }
  // } else {
  //   state
  //     .focused_container()
  //     .context("No focused container.")?
  //     .workspace()
  //     .context("No workspace.")?
  // };
  //
  let (target_workspace, target_container) =
    if let Some(target) = target_parent {
      if target.is_workspace() {
        (target.workspace().context("No workspace.")?, target.clone())
      } else {
        let parent = target.parent().context("No parent.")?;
        (parent.workspace().context("No workspace.")?, target.clone())
      }
    } else {
      let focused =
        state.focused_container().context("No focused container.")?;
      (
        focused.workspace().context("No workspace.")?,
        focused.clone(),
      )
    };

  match target_workspace.tiling_layout() {
    TilingLayout::Manual {
      tiling_direction: _,
    } => add_manual_window(
      target_parent.clone(),
      window_state,
      target_container,
      target_workspace,
    ),
    TilingLayout::MasterStack { master_ratio } => add_master_stack_window(
      target_parent.clone(),
      window_state,
      target_workspace,
      target_container,
      config,
      state,
    ),
    TilingLayout::Dwindle => add_dwindle_window(
      target_parent.clone(),
      window_state,
      target_workspace,
      target_container,
      config,
      state,
    ),
    TilingLayout::Grid => add_grid_window(
      target_parent.clone(),
      window_state,
      target_workspace,
      target_container,
      config,
      state,
    ),
  }
}

fn add_manual_window(
  target_parent: Option<Container>,
  window_state: &WindowState,
  target_container: Container,
  target_workspace: Workspace,
) -> anyhow::Result<(Container, usize)> {
  if let Some(target) = target_parent {
    return Ok((target.clone(), 0));
  }
  // For tiling windows, try to find a suitable tiling window to insert
  // next to.
  if *window_state == WindowState::Tiling {
    let sibling = match target_container {
      Container::TilingWindow(_) => Some(target_container),
      _ => target_workspace
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
    target_workspace.clone().into(),
    target_workspace.child_count(),
  ))
}

fn add_master_stack_window(
  target_parent: Option<Container>,
  window_state: &WindowState,
  target_workspace: Workspace,
  target_container: Container,
  config: &UserConfig,
  state: &mut WmState,
) -> anyhow::Result<(Container, usize)> {
  let child_c = target_workspace.child_count();
  let child_count = target_workspace.children().iter().count();
  assert_eq!(child_c, child_count);
  if child_count == 0 {
    Ok((target_workspace.clone().into(), 0))
  } else if child_count == 1 {
    // Create a vertical split container for the stack
    let stack_container = SplitContainer::new(
      TilingDirection::Vertical,
      config.value.gaps.clone(),
    );
    {
      let master_containers = target_workspace.borrow_children_mut();
      let master_container = master_containers.front().unwrap();

      // master_container
      //   .as_tiling_container()?
      //   .set_tiling_size(master_ratio);

      state
        .pending_sync
        .queue_container_to_redraw(master_container.clone());
    }

    attach_container(
      &stack_container.clone().into(),
      &target_workspace.clone().into(),
      None,
    )?;
    Ok((stack_container.clone().into(), 0))
  } else if child_count == 2 {
    let children = target_workspace.children();
    let stack_container = children.back().context("No children.")?;

    Ok((stack_container.clone(), 0))
  } else {
    assert!(false);
    // If there are no children, just append to the workspace.
    Ok((target_workspace.clone().into(), 0))
  }
}

fn add_dwindle_window(
  target_parent: Option<Container>,
  window_state: &WindowState,
  target_workspace: Workspace,
  target_container: Container,
  config: &UserConfig,
  state: &mut WmState,
) -> anyhow::Result<(Container, usize)> {
  let child_c = target_workspace.child_count();
  let child_count = target_workspace.children().iter().count();
  assert_eq!(child_c, child_count);

  if child_count == 0 {
    Ok((target_workspace.clone().into(), 0))
  } else if child_count == 1 {
    // Create a vertical split container for the stack
    let new_container = SplitContainer::new(
      TilingDirection::Vertical,
      config.value.gaps.clone(),
    );
    {
      let master_containers = target_workspace.borrow_children_mut();
      let only_container = master_containers.front().unwrap();
      state
        .pending_sync
        .queue_container_to_redraw(only_container.clone());
    }
    attach_container(
      &new_container.clone().into(),
      &target_workspace.clone().into(),
      None,
    )?;
    Ok((new_container.clone().into(), 0))
  } else if child_count == 2 {
    let children = target_workspace.borrow_children_mut();
    let back_container = children.back().context("No children.")?;
    let back_clone = back_container.clone();
    // Release the borrow by dropping 'children'
    drop(children);

    // Now work with the cloned container
    let mut current_container = back_clone;

    while current_container.children().iter().count() > 1 {
      // Create a new borrow scope
      let next_container = {
        let children = current_container.borrow_children_mut();
        let back = children.back().context("No children.")?;
        back.clone() // Clone it so we can drop the borrow
      };
      current_container = next_container;
    }

    let current_child_count = current_container.children().iter().count();
    if current_child_count == 0 {
      Ok((target_workspace.clone().into(), 0))
    } else if current_child_count == 1 {
      let new_split_direction = current_container
        .as_direction_container()?
        .tiling_direction()
        .inverse();
      let split_container = SplitContainer::new(
        new_split_direction,
        config.value.gaps.clone(),
      );
      state
        .pending_sync
        .queue_container_to_redraw(current_container.clone());
      attach_container(
        &split_container.clone().into(),
        &current_container.clone().into(),
        None,
      )?;
      Ok((split_container.clone().into(), 0))
    } else {
      assert!(false);
      Err(anyhow::anyhow!("Unexpected child count"))
    }
  } else {
    assert!(false);
    // If there are no children, just append to the workspace.
    Err(anyhow::anyhow!("Unexpected child count"))
  }
}

fn add_grid_window(
  target_parent: Option<Container>,
  window_state: &WindowState,
  target_workspace: Workspace,
  target_container: Container,
  config: &UserConfig,
  state: &mut WmState,
) -> anyhow::Result<(Container, usize)> {
  let child_c = target_workspace.child_count();
  let child_count = target_workspace.children().iter().count();
  assert_eq!(child_c, child_count);

  let current_container = target_workspace.clone();
  let mut result = None;
  current_container.children().iter().for_each(|container| {
    if container.child_count() < child_count {
      result = Some((container.clone().into(), 0));
      return;
    }
  });

  if let Some(result) = result {
    return Ok(result);
  }

  let vert_container = SplitContainer::new(
    TilingDirection::Vertical,
    config.value.gaps.clone(),
  );
  {
    let master_containers = target_workspace.borrow_children_mut();
    state
      .pending_sync
      .queue_containers_to_redraw(master_containers.clone());
  }

  attach_container(
    &vert_container.clone().into(),
    &target_workspace.clone().into(),
    None,
  )?;
  Ok((vert_container.clone().into(), 0))
}
