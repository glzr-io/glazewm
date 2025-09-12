use tracing::info;
use wm_common::{try_warn, WindowState};
use wm_platform::NativeWindow;

use crate::{
  commands::{
    container::set_focused_descendant, window::update_window_state,
  },
  traits::WindowGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_window_minimized(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(native_window);

  // Update the window's state to be minimized.
  if let Some(window) = found_window {
    let is_minimized = try_warn!(window.native().is_minimized());

    window.update_native_properties(|properties| {
      properties.is_minimized = is_minimized;
    });

    if is_minimized && window.state() != WindowState::Minimized {
      info!("Window minimized: {window}");

      let window = update_window_state(
        window.clone(),
        WindowState::Minimized,
        state,
        config,
      )?;

      // Focus should be reassigned after a window has been minimized.
      if let Some(focus_target) = state.focus_target_after_removal(&window)
      {
        set_focused_descendant(&focus_target, None);
        state.pending_sync.queue_focus_change();
        state.unmanaged_or_minimized_timestamp =
          Some(std::time::Instant::now());
      }
    }
  }

  Ok(())
}
