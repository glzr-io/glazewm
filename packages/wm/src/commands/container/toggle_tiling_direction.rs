use anyhow::Context;
use wm_common::{TilingDirection, WmEvent};

use super::{flatten_split_container, wrap_in_container};
use crate::{
  models::{
    enclosing_group, Container, DirectionContainer, SplitContainer,
    TilingContainer, TilingWindow,
  },
  traits::{CommonGetters, TilingDirectionGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn toggle_tiling_direction(
  container: Container,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let direction_container = match container {
    Container::TilingWindow(tiling_window) => {
      toggle_tile_direction(tile_of_window(tiling_window), config)
    }
    Container::Workspace(workspace) => {
      workspace
        .set_tiling_direction(workspace.tiling_direction().inverse());

      Ok(workspace.into())
    }
    // Can only toggle tiling direction from a tiling window or workspace.
    _ => return Ok(()),
  }?;

  state.emit_event(WmEvent::TilingDirectionChanged {
    direction_container: direction_container.to_dto()?,
    new_tiling_direction: direction_container.tiling_direction(),
  });

  Ok(())
}

/// Gets the container that a window occupies in the tiling layout.
///
/// For a page of a group, this is the group itself.
fn tile_of_window(tiling_window: TilingWindow) -> TilingContainer {
  enclosing_group(&tiling_window)
    .map_or_else(|| tiling_window.into(), Into::into)
}

/// Toggles the tiling direction of a tile in the layout.
fn toggle_tile_direction(
  tile: TilingContainer,
  config: &UserConfig,
) -> anyhow::Result<DirectionContainer> {
  let parent = tile
    .direction_container()
    .context("No direction container.")?;

  // If the tile is an only child, then either change the tiling
  // direction of its parent workspace or flatten its parent split
  // container.
  if tile.tiling_siblings().count() == 0 {
    return match parent {
      DirectionContainer::Workspace(workspace) => {
        workspace
          .set_tiling_direction(workspace.tiling_direction().inverse());

        Ok(workspace.into())
      }
      DirectionContainer::Split(split_container) => {
        flatten_split_container(split_container.clone())?;

        tile
          .direction_container()
          .context("No direction container.")
      }
    };
  }

  // Create a new split container to wrap the tile.
  let split_container = SplitContainer::new(
    parent.tiling_direction().inverse(),
    config.value.gaps.clone(),
  );

  wrap_in_container(&split_container, &parent.into(), &[tile])?;

  Ok(split_container.into())
}

pub fn set_tiling_direction(
  container: Container,
  state: &mut WmState,
  config: &UserConfig,
  tiling_direction: &TilingDirection,
) -> anyhow::Result<()> {
  let direction_container = container
    .direction_container()
    .context("No direction container.")?;

  if direction_container.tiling_direction() == *tiling_direction {
    Ok(())
  } else {
    toggle_tiling_direction(container, state, config)
  }
}
