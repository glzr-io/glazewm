use crate::{
  containers::WindowContainer, user_config::UserConfig,
  windows::traits::WindowGetters, wm_state::WmState,
};

pub fn apply_window_effects(
  window: WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let inactive_border_color =
    config.value.window_effects.other_windows.border_color;

  // Clear old window border.
  if let Some(active_border_window) = &state.active_border_window {
    active_border_window
      .native()
      .set_border_color(inactive_border_color.as_ref())?;
  }

  let active_border_color =
    config.value.window_effects.focused_window.border_color;

  // Set new window border.
  window
    .native()
    .set_border_color(active_border_color.as_ref())?;

  state.active_border_window = Some(window);

  Ok(())
}
