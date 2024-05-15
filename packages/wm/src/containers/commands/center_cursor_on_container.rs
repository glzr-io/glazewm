use crate::common::platform::Platform;
use crate::containers::traits::PositionGetters;
use crate::containers::Container;
use crate::user_config::UserConfig;
use anyhow::{Error, Result};

pub fn center_cursor_on_container(
  target: &Container,
  config: &UserConfig,
) -> Result<(), Error> {
  if !config.value.general.cursor_follows_focus
  {
    return Ok(());
  }

  let center_x = target.x()? + (target.width()? / 2);
  let center_y = target.y()? + (target.height()? / 2);

  Platform::set_cursor_pos(center_x, center_y)?;

  Ok(())
}
