use anyhow::Context;
use wm_common::{TilingLayout, WmEvent};

use crate::{
  models::{Container, DirectionContainer},
  traits::{CommonGetters, TilingLayoutGetters},
  wm_state::WmState,
};

/// Toggles the tiling layout of the subject's nearest layout container.
pub fn toggle_tiling_layout(
  container: &Container,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let Some(layout_container) = layout_container_for(container) else {
    return Ok(());
  };

  let target_layout = layout_container.tiling_layout().inverse();
  update_tiling_layout(&layout_container, target_layout, state)
}

/// Sets the tiling layout of the subject's nearest layout container.
pub fn set_tiling_layout(
  container: &Container,
  tiling_layout: &TilingLayout,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let Some(layout_container) = layout_container_for(container) else {
    return Ok(());
  };

  update_tiling_layout(&layout_container, tiling_layout.clone(), state)
}

/// Gets the layout container targeted by a tiling layout command.
fn layout_container_for(
  container: &Container,
) -> Option<DirectionContainer> {
  match container {
    Container::Workspace(workspace) => Some(workspace.clone().into()),
    Container::Split(split) => Some(split.clone().into()),
    Container::TilingWindow(window) => window
      .parent()
      .and_then(|parent| parent.as_direction_container().ok()),
    _ => None,
  }
}

/// Applies a layout change and queues its native synchronization.
fn update_tiling_layout(
  layout_container: &DirectionContainer,
  tiling_layout: TilingLayout,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if !apply_tiling_layout(layout_container, tiling_layout.clone()) {
    return Ok(());
  }

  let workspace = layout_container
    .workspace()
    .context("Layout container has no workspace.")?;

  state
    .pending_sync
    .queue_container_to_redraw(layout_container.clone())
    .queue_workspace_to_reorder(workspace);

  state.emit_event(WmEvent::TilingLayoutChanged {
    direction_container: layout_container.to_dto()?,
    new_tiling_layout: tiling_layout,
  });

  Ok(())
}

/// Applies a tiling layout and returns whether it changed.
fn apply_tiling_layout(
  layout_container: &DirectionContainer,
  tiling_layout: TilingLayout,
) -> bool {
  if layout_container.tiling_layout() == tiling_layout {
    return false;
  }

  layout_container.set_tiling_layout(tiling_layout);
  true
}

#[cfg(test)]
mod tests {
  use wm_common::TilingLayout;

  use super::{apply_tiling_layout, layout_container_for};
  use crate::{
    models::{Container, NonTilingWindow, SplitContainer, TilingWindow},
    traits::{CommonGetters, TilingDirectionGetters, TilingLayoutGetters},
  };

  #[test]
  fn tiling_window_targets_its_immediate_parent() {
    let window = TilingWindow::mock().call();
    let split = SplitContainer::mock()
      .tiling_containers(vec![window.clone().into()])
      .call();

    let target = layout_container_for(&window.into()).unwrap();
    assert_eq!(target.id(), split.id());
  }

  #[test]
  fn non_tiling_window_does_not_target_workspace() {
    let window = NonTilingWindow::mock().call();
    assert!(layout_container_for(&Container::from(window)).is_none());
  }

  #[test]
  fn applying_layout_is_idempotent_and_preserves_direction() {
    let split = SplitContainer::mock().call();
    let layout_container = split.clone().into();
    let direction = split.tiling_direction();

    assert!(apply_tiling_layout(
      &layout_container,
      TilingLayout::Accordion
    ));
    assert_eq!(split.tiling_layout(), TilingLayout::Accordion);
    assert_eq!(split.tiling_direction(), direction);
    assert!(!apply_tiling_layout(
      &layout_container,
      TilingLayout::Accordion
    ));
  }
}
