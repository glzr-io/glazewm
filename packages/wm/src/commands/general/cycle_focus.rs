use anyhow::Context;
use wm_common::WindowState;

use crate::{
  commands::container::set_focused_descendant,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Cycles focus through windows of different states. In order, this will
/// change from tiling -> floating -> fullscreen -> minimized, then back to
/// tiling.
///
/// Does nothing if a workspace is focused.
pub fn cycle_focus(
  omit_floating: bool,
  omit_fullscreen: bool,
  omit_minimized: bool,
  omit_tiling: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_container =
    state.focused_container().context("No focused container.")?;

  if let Ok(window) = focused_container.as_window_container() {
    let workspace = window.workspace().context("No workspace.")?;

    let current = window.state();
    let mut next = next_state(&current, config);

    loop {
      // Break if we have cycled back to the current state.
      if current.is_same_state(&next) {
        break;
      }

      // Skip the next state if it is to be omitted.
      if (omit_floating && matches!(next, WindowState::Floating(_)))
        || omit_fullscreen && matches!(next, WindowState::Fullscreen(_))
        || omit_minimized && matches!(next, WindowState::Minimized)
        || omit_tiling && matches!(next, WindowState::Tiling)
      {
        next = next_state(&next, config);
        continue;
      }

      // Get window that matches the next state.
      let window_of_type = workspace
        .descendant_focus_order()
        .filter_map(|descendant| descendant.as_window_container().ok())
        .find(|descendant| {
          matches!(
            (descendant.state(), &next),
            (WindowState::Floating(_), WindowState::Floating(_))
              | (WindowState::Fullscreen(_), WindowState::Fullscreen(_))
              | (WindowState::Minimized, WindowState::Minimized)
              | (WindowState::Tiling, WindowState::Tiling)
          )
        });

      if let Some(window) = window_of_type {
        set_focused_descendant(&window.into(), None);
        state.pending_sync.queue_focus_change();
        break;
      }

      next = next_state(&next, config);
    }
  }

  Ok(())
}

fn next_state(
  current_state: &WindowState,
  config: &UserConfig,
) -> WindowState {
  match current_state {
    WindowState::Floating(_) => WindowState::Fullscreen(
      config
        .value
        .window_behavior
        .state_defaults
        .fullscreen
        .clone(),
    ),
    WindowState::Fullscreen(_) => WindowState::Minimized,
    WindowState::Minimized => WindowState::Tiling,
    WindowState::Tiling => WindowState::Floating(
      config.value.window_behavior.state_defaults.floating.clone(),
    ),
  }
}
