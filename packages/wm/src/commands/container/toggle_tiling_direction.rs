use anyhow::Context;
use wm_common::{TilingDirection, WmEvent};

use super::{flatten_split_container, wrap_in_split_container};
use crate::{
  models::{Container, DirectionContainer, SplitContainer, TilingWindow},
  traits::{CommonGetters, TilingDirectionGetters, TilingLayoutGetters},
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
      toggle_window_direction(tiling_window, config)
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

fn toggle_window_direction(
  tiling_window: TilingWindow,
  config: &UserConfig,
) -> anyhow::Result<DirectionContainer> {
  let parent = tiling_window
    .direction_container()
    .context("No direction container.")?;

  // If the window is an only child, then either change the tiling
  // direction of its parent workspace or flatten its parent split
  // container.
  if tiling_window.tiling_siblings().count() == 0 {
    return match parent {
      DirectionContainer::Workspace(workspace) => {
        workspace
          .set_tiling_direction(workspace.tiling_direction().inverse());

        Ok(workspace.into())
      }
      DirectionContainer::Split(split_container) => {
        toggle_single_child_split(&tiling_window, &split_container)
      }
    };
  }

  // Create a new split container to wrap the window.
  let split_container = SplitContainer::new(
    parent.tiling_direction().inverse(),
    config.value.gaps.clone(),
  );

  wrap_in_split_container(
    &split_container,
    &parent.into(),
    &[tiling_window.into()],
  )?;

  Ok(split_container.into())
}

/// Toggles a single-child split without discarding a distinct layout.
fn toggle_single_child_split(
  tiling_window: &TilingWindow,
  split_container: &SplitContainer,
) -> anyhow::Result<DirectionContainer> {
  let grandparent = split_container
    .parent()
    .and_then(|parent| parent.as_direction_container().ok())
    .context("Split container has no direction container parent.")?;

  let target_direction = split_container.tiling_direction().inverse();
  let can_flatten = split_container.tiling_layout()
    == grandparent.tiling_layout()
    && target_direction == grandparent.tiling_direction();

  if can_flatten {
    flatten_split_container(split_container.clone())?;
    tiling_window
      .direction_container()
      .context("No direction container.")
  } else {
    split_container.set_tiling_direction(target_direction);
    Ok(split_container.clone().into())
  }
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

#[cfg(test)]
mod tests {
  use wm_common::{TilingDirection, TilingLayout};

  use super::toggle_single_child_split;
  use crate::{
    models::{SplitContainer, TilingWindow, Workspace},
    traits::{CommonGetters, TilingDirectionGetters, TilingLayoutGetters},
  };

  #[test]
  fn toggling_direction_preserves_distinct_accordion_split() {
    let window = TilingWindow::mock().call();
    let split = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![window.clone().into()])
      .call();
    split.set_tiling_layout(TilingLayout::Accordion);

    let _workspace = Workspace::mock()
      .tiling_direction(TilingDirection::Horizontal)
      .tiling_containers(vec![
        split.clone().into(),
        TilingWindow::mock().call().into(),
      ])
      .call();

    let toggled = toggle_single_child_split(&window, &split)
      .expect("Direction toggle should succeed.");

    assert_eq!(toggled.id(), split.id());
    assert!(!split.is_detached());
    assert_eq!(split.tiling_direction(), TilingDirection::Horizontal);
    assert_eq!(split.tiling_layout(), TilingLayout::Accordion);
  }
}
