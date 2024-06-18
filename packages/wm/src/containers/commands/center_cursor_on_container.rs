use crate::{
  common::platform::Platform, containers::traits::PositionGetters,
  containers::Container, user_config::UserConfig,
};

pub fn center_cursor_on_container(
  target: Container,
  config: &UserConfig,
) -> anyhow::Result<()> {
  match config.value.general.cursor_jump.enabled {
    false => Ok(()),
    true => {
      let center = target.to_rect()?.center_point();
      Platform::set_cursor_pos(center.x, center.y)
    }
  }
}
