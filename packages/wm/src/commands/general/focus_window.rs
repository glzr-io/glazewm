use anyhow;
use regex::Regex;
use tracing::info;

use crate::{
    commands::{
        container::set_focused_descendant,
        workspace::focus_workspace,
    },
    models::WorkspaceTarget,
    traits::{WindowGetters, CommonGetters},
    wm_state::WmState,
    user_config::UserConfig,
};

/// Focus a window where the title matches a given regex pattern.
///
/// If the window is in a different workspace, it will switch to that workspace first.
/// Does nothing if no matching window is found.
pub fn focus_window(
    title_regex: &Vec<String>,
    state: &mut WmState,
    config: &UserConfig,
) -> anyhow::Result<()> {
    let pattern_str = title_regex.join(" ");
    // Compile the regex pattern
    let pattern = Regex::new(&pattern_str)
        .map_err(|e| anyhow::anyhow!("Invalid regex pattern: {}", e))?;

    // Get all windows and find the first one with a matching title
    let windows = state.windows();

    for window in windows {
        let title = window.native().title()?;

        if pattern.is_match(&title) {
            info!("Focusing window with title matching pattern '{}': '{}'", pattern_str, title);

            // Get the window's workspace
            let window_workspace = window.workspace()
                .ok_or_else(|| anyhow::anyhow!("Window has no workspace"))?;

            // Get the current workspace
            let current_workspace = state.focused_container()
                .and_then(|focused| focused.workspace())
                .ok_or_else(|| anyhow::anyhow!("No workspace is currently focused"))?;

            // If the window is in a different workspace, switch to that workspace first
            if window_workspace.id() != current_workspace.id() {
                focus_workspace(
                    WorkspaceTarget::Name(window_workspace.config().name.clone()),
                    state,
                    config
                )?;
            }

            // Set the matching window as focused
            set_focused_descendant(&window.clone().into(), None);
            state.pending_sync.queue_focus_change().queue_cursor_jump();

            return Ok(());
        }
    }

    info!("No window found with title matching pattern: '{}'", pattern_str);
    Ok(())
}
