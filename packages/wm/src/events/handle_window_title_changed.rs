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

    try_warn!(window.native().refresh_title());

    // Run window rules for title change events.
    run_window_rules(
      window,
      &WindowRuleEvent::TitleChange,
      state,
      config,
    )?;
  }

  Ok(())
}
