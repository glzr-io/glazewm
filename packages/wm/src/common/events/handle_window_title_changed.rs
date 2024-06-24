use tracing::info;

use crate::{
  common::platform::NativeWindow,
  user_config::{UserConfig, WindowRuleEvent},
  windows::{commands::run_window_rules, traits::WindowGetters},
  wm_state::WmState,
};

pub fn handle_window_title_changed(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  if let Some(window) = found_window {
    // TODO: Log window details.
    info!("Window title changed");

    // Run window rules for title change events.
    window.native().refresh_title()?;
    run_window_rules(window, WindowRuleEvent::TitleChange, state, config)?;
  }

  Ok(())
}
