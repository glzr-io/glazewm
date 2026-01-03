use anyhow::Context;
use serde::Deserialize;
use std::fs::File;
use std::path::PathBuf;
use tracing::info;
use wm_common::{DisplayState, WindowRuleEvent, WmEvent};
use wm_platform::{NativeWindow, Platform};

use crate::{
  commands::{
    container::set_focused_descendant, window::run_window_rules,
    workspace::focus_workspace, window::move_window_to_workspace,
  },
  models::WorkspaceTarget,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_window_focused(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(native_window);
  let focused_container =
    state.focused_container().context("No focused container.")?;

  // Update the focus sync state. If the OS focused window is not same as
  // the WM's focused container, then the focus is not synced.
  state.is_focus_synced = match focused_container.as_window_container() {
    Ok(window) => *window.native() == *native_window,
    _ => Platform::desktop_window() == *native_window,
  };

  // Handle overriding focus on close/minimize. After a window is closed
  // or minimized, the OS or the closed application might automatically
  // switch focus to a different window. To force focus to go to the WM's
  // target focus container, we reassign any focus events 100ms after
  // close/minimize. This will cause focus to briefly flicker to the OS
  // focus target and then to the WM's focus target.
  if should_override_focus(state) {
    state.pending_sync.queue_focus_change();
    return Ok(());
  }

  // Ignore the focus event if window is being hidden by the WM.
  if let Some(window) = &found_window {
    if window.display_state() == DisplayState::Hiding {
      return Ok(());
    }
  }

  // Focus effect should be updated for any change in focus that shouldn't
  // be overwritten. Focus here is either:
  //  1. WM's focus container (window).
  //  2. WM's focus container (workspace - i.e. the desktop window).
  //  3. An ignored window.
  //  4. A window that received manual focus.
  state.pending_sync.queue_focused_effect_update();

  if let Some(window) = found_window {
    let workspace = window.workspace().context("No workspace")?;

    // Native focus has been synced to the WM's focused container.
    if focused_container == window.clone().into() {
      state.is_focus_synced = true;
      state.pending_sync.queue_workspace_to_reorder(workspace);
      return Ok(());
    }

    info!("Window manually focused: {window}");

    // Check mapping file for a saved target for this window. If found,
    // move the window to the saved workspace and remove the entry.
    #[derive(Deserialize, serde::Serialize, Clone)]
    struct AppWorkspaceEntry {
      handle: isize,
      process_name: Option<String>,
      class_name: Option<String>,
      title: Option<String>,
      workspace: String,
    }

        // Only consult mapping if persistence is enabled.
        if config.value.general.persists_process_location {
          let mapping_path = config
            .path
            .parent()
            .map(|p| p.join("glazewm_apps_workspaces.json"))
            .unwrap_or_else(|| PathBuf::from("glazewm_apps_workspaces.json"));

          if mapping_path.exists() {
            if let Ok(file) = File::open(&mapping_path) {
              if let Ok(mut entries) = serde_json::from_reader::<_, Vec<AppWorkspaceEntry>>(file) {
                // Determine identifying values for the focused window.
                let native = window.native().clone();
                let proc = native.process_name().ok();
                let class = native.class_name().ok();
                let title = native.title().ok();

                if let Some(pos) = entries.iter().position(|e| {
                  e.handle == native.handle
                    || e
                      .process_name
                      .as_ref()
                      .map(|p| proc.as_ref().map(|s| s == p).unwrap_or(false))
                      .unwrap_or(false)
                    || e
                      .title
                      .as_ref()
                      .map(|t| title.as_ref().map(|s| s.contains(t)).unwrap_or(false))
                      .unwrap_or(false)
                }) {
                  let entry = entries.remove(pos);

                  // Write back remaining entries atomically.
                  let _ = (|| -> anyhow::Result<()> {
                    let tmp = mapping_path.with_extension("tmp");
                    let file = std::fs::File::create(&tmp)?;
                    serde_json::to_writer_pretty(file, &entries)?;
                    std::fs::rename(tmp, &mapping_path)?;
                    Ok(())
                  })();

                  // Move the window to the saved workspace.
                  let _ = move_window_to_workspace(
                    window.clone(),
                    WorkspaceTarget::Name(entry.workspace.clone()),
                    state,
                    config,
                  );
                }
              }
            }
          }
        }

    // Handle focus events from windows on hidden workspaces. For example,
    // if Discord is forcefully shown by the OS when it's on a hidden
    // workspace, switch focus to Discord's workspace.
    if window.display_state() == DisplayState::Hidden {
      info!("Focusing off-screen window: {window}");

      focus_workspace(
        WorkspaceTarget::Name(workspace.config().name),
        state,
        config,
      )?;
    }

    // Update the WM's focus state.
    set_focused_descendant(&window.clone().into(), None);

    // Run window rules for focus events.
    run_window_rules(
      window.clone(),
      &WindowRuleEvent::Focus,
      state,
      config,
    )?;

    state.is_focus_synced = true;
    state.pending_sync.queue_workspace_to_reorder(workspace);

    // Broadcast the focus change event.
    state.emit_event(WmEvent::FocusChanged {
      focused_container: window.to_dto()?,
    });
  }

  Ok(())
}

/// Returns true if focus should be reassigned to the WM's focus container.
fn should_override_focus(state: &WmState) -> bool {
  let has_recent_unmanage = state
    .unmanaged_or_minimized_timestamp
    .is_some_and(|time| time.elapsed().as_millis() < 100);

  has_recent_unmanage && !state.is_focus_synced
}
