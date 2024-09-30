use crate::{
  containers::{traits::CommonGetters, Container},
  windows::traits::WindowGetters,
};

pub fn set_title_bar_visibility(
  title_bar_is_visible: bool,
  subject_container: Container,
) -> anyhow::Result<()> {
  let window = subject_container.as_window_container()?;
  window
    .native()
    .set_title_bar_visibility(title_bar_is_visible)?;

  Ok(())
}
