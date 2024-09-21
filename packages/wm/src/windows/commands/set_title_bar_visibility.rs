use crate::{
  common::{commands::platform_sync, DisplayState},
  containers::{
    traits::{CommonGetters, PositionGetters},
    Container,
  },
  user_config::UserConfig,
  windows::traits::WindowGetters,
  wm_state::WmState,
};

pub fn set_title_bar_visibility(
  title_bar_is_visible: bool,
  subject_container: Container,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let window = subject_container.as_window_container()?;
  window
    .native()
    .set_title_bar_visibility(title_bar_is_visible)?;

  // SetWindowPos needs to be called to make sure that the window is
  // updated.
  let window_is_visible = match window.display_state() {
    DisplayState::Showing | DisplayState::Shown => true,
    _ => false,
  };
  window.native().set_position(
    &window.state(),
    &window.to_rect()?,
    window_is_visible,
    window.has_pending_dpi_adjustment(),
  )?;

  state
    .pending_sync
    .containers_to_redraw
    .push(subject_container);
  platform_sync(state, config)?;

  Ok(())
}
