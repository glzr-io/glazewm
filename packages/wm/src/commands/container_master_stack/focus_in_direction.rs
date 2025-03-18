use anyhow::Context;
use wm_common::{Direction, TilingDirection, WindowState};

use crate::{
  commands::container::set_focused_descendant,
  models::{Container, TilingContainer, TilingLayout, Workspace},
  traits::{CommonGetters, TilingDirectionGetters, WindowGetters},
  wm_state::WmState,
};

pub fn focus_in_direction(
  origin_container: &Container,
  direction: &Direction,
  workspace: &Workspace,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if let Ok(Some(focus_target)) =
    window_focus_target(origin_container, workspace, direction)
      .map_or_else(
        || workspace_focus_target(origin_container, direction, state),
        |container| Ok(Some(container)),
      )
  {
    set_focused_descendant(&focus_target, None);
    state.pending_sync.queue_focus_change().queue_cursor_jump();
  }
  Ok(())
}

pub fn window_focus_target(
  origin_container: &Container,
  workspace: &Workspace,
  direction: &Direction,
) -> Option<Container> {
  let master_window = match workspace.tiling_layout() {
    TilingLayout::MasterStack { master_window } => master_window,
    _ => None,
  };
  let is_master = master_window.as_ref()
    .map_or(false, |master| master.id() == origin_container.id());

  if is_master {
    match direction {
      Direction::Right => {
        // Focus window stack
        todo!()
      }
      _ => {}
    }
  }
  println!("is_master: {}", is_master);
  println!("direction: {:?}", direction);
  println!(
    "workspace: {:#?}",
    workspace.next_siblings().collect::<Vec<_>>()
  );
  // We must be looking at a stack window
  let focus_target = match direction {
    Direction::Up => origin_container
      .prev_siblings()
      .find_map(|c| c.as_tiling_container().ok()),
    Direction::Down => origin_container
      .next_siblings()
      .find_map(|c| c.as_tiling_container().ok()),
    Direction::Left => master_window.map(Into::into),
    _ => None,
  };
  println!("focus_target: {:?}", focus_target);
  focus_target.map(Into::into)
}

// TODO - c+p this for now
fn workspace_focus_target(
  origin_container: &Container,
  direction: &Direction,
  state: &WmState,
) -> anyhow::Result<Option<Container>> {
  let monitor = origin_container.monitor().context("No monitor.")?;

  let target_workspace = state
    .monitor_in_direction(&monitor, direction)?
    .and_then(|monitor| monitor.displayed_workspace());

  let focused_fullscreen = target_workspace
    .as_ref()
    .and_then(|workspace| workspace.descendant_focus_order().next())
    .filter(|focused| match focused {
      Container::NonTilingWindow(window) => {
        matches!(window.state(), WindowState::Fullscreen(_))
      }
      _ => false,
    });

  let focus_target = focused_fullscreen
    .or_else(|| {
      target_workspace.as_ref().and_then(|workspace| {
        workspace
          .descendant_in_direction(&direction.inverse())
          .map(Into::into)
      })
    })
    .or(target_workspace.map(Into::into));

  Ok(focus_target)
}
