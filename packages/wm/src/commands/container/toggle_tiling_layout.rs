use anyhow::Context;
use wm_common::TilingLayout;

use crate::{
  models::{Container, DirectionContainer},
  traits::{CommonGetters, TilingDirectionGetters},
  wm_state::WmState,
};

/// Toggles the layout of the direction container (split container or
/// workspace) that the given container is in.
pub fn toggle_tiling_layout(
  container: &Container,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let Some(direction_container) = layout_target(container)? else {
    return Ok(());
  };

  set_layout(
    &direction_container,
    state,
    direction_container.tiling_layout().toggled(),
  );

  Ok(())
}

/// Sets the layout of the direction container (split container or
/// workspace) that the given container is in.
pub fn set_tiling_layout(
  container: &Container,
  state: &mut WmState,
  tiling_layout: &TilingLayout,
) -> anyhow::Result<()> {
  let Some(direction_container) = layout_target(container)? else {
    return Ok(());
  };

  if direction_container.tiling_layout() != *tiling_layout {
    set_layout(&direction_container, state, tiling_layout.clone());
  }

  Ok(())
}

/// Gets the direction container whose layout should be changed. Only
/// tiling windows and workspaces have a meaningful target.
fn layout_target(
  container: &Container,
) -> anyhow::Result<Option<DirectionContainer>> {
  match container {
    Container::TilingWindow(_) | Container::Workspace(_) => Ok(Some(
      container
        .direction_container()
        .context("No direction container.")?,
    )),
    _ => Ok(None),
  }
}

fn set_layout(
  direction_container: &DirectionContainer,
  state: &mut WmState,
  tiling_layout: TilingLayout,
) {
  direction_container.set_tiling_layout(tiling_layout);

  // Redraw the container and all of its descendants.
  state
    .pending_sync
    .queue_container_to_redraw(direction_container.clone());
}
