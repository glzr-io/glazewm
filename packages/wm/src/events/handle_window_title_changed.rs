use tracing::info;
use wm_common::{try_warn, WindowRuleEvent};
use wm_platform::NativeWindow;

use crate::{
  commands::window::run_window_rules, traits::WindowGetters,
  user_config::UserConfig, wm_state::WmState,
};

pub fn handle_window_title_changed(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(native_window);

  if let Some(window) = found_window {
    info!("Window title changed: {window}");

    let title = try_warn!(window.native().title());

    window.update_native_properties(|properties| {
      properties.title = title;
    });

    // Native macOS tab switches reuse the same `AXUIElement` but
    // change the underlying `CGWindowID`. Detect this and queue a
    // redraw so the newly active tab gets positioned correctly.
    #[cfg(target_os = "macos")]
    {
      use wm_platform::NativeWindowExtMacOs;

      if let Ok(current_id) = native_window.current_window_id() {
        if current_id != native_window.id() {
          info!(
            "Tab switch detected for {window}: {} -> {}",
            native_window.id().0,
            current_id.0,
          );
          state.pending_sync.queue_container_to_redraw(window.clone());
        }
      }
    }

    run_window_rules(
      window,
      &WindowRuleEvent::TitleChange,
      state,
      config,
    )?;
  }

  Ok(())
}
