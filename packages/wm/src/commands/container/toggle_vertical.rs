use anyhow::Context;
use tracing::info;
use wm_common::TilingDirection;

use super::{
  flatten_split_container, move_container_within_tree,
  wrap_in_split_container,
};
use crate::{
  models::{Container, SplitContainer, TilingWindow},
  traits::{CommonGetters, TilingDirectionGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Toggles vertical tiling with the neighbor window.
///
/// This command takes the focused window and groups it vertically with its
/// left neighbor (or right neighbor if no left neighbor exists).
pub fn toggle_vertical(
  container: Container,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("Toggle vertical called");

  // Only works with tiling windows
  let tiling_window = match container {
    Container::TilingWindow(window) => {
      info!("Container is a tiling window: {}", window.id());
      window
    }
    _ => {
      info!("Container is not a tiling window, skipping");
      return Ok(());
    }
  };

  // Get the parent container
  let parent = tiling_window
    .parent()
    .context("No parent container.")?;

  info!(
    "Parent container: {} (type: {})",
    parent.id(),
    if parent.as_workspace().is_some() {
      "Workspace"
    } else if parent.as_split().is_some() {
      "Split"
    } else {
      "Other"
    }
  );

  // Check if both windows are already in a vertical split together
  if let Some(split_parent) = parent.as_split() {
    info!(
      "Parent is a split container with direction: {:?}, child count: {}",
      split_parent.tiling_direction(),
      split_parent.child_count()
    );
    
    if split_parent.tiling_direction() == TilingDirection::Vertical
      && split_parent.child_count() == 2
    {
      // They're already grouped vertically, so flatten back to horizontal
      info!("Already in vertical split, flattening back to horizontal");
      
      // Get the grandparent before flattening
      let grandparent = split_parent.parent().context("No grandparent.")?;
      
      flatten_split_container(split_parent.clone())?;
      
      // Queue grandparent's children for redraw
      state
        .pending_sync
        .queue_containers_to_redraw(grandparent.tiling_children());
      
      return Ok(());
    }

    if split_parent.tiling_direction() == TilingDirection::Vertical
      && split_parent.child_count() > 2
    {
      // Parent is vertical with 3+ children — move this window out
      // to the grandparent so it becomes a horizontal sibling.
      info!(
        "Parent is vertical with {} children, moving window out to grandparent",
        split_parent.child_count()
      );

      let grandparent =
        split_parent.parent().context("No grandparent.")?;
      let target_index = split_parent.index() + 1;

      move_container_within_tree(
        &tiling_window.clone().into(),
        &grandparent,
        target_index,
        state,
      )?;

      state
        .pending_sync
        .queue_containers_to_redraw(grandparent.tiling_children());

      return Ok(());
    }
  }

  // Get the neighbor window (prefer left, fallback to right)
  let (neighbor, neighbor_on_left) = find_neighbor_window(&tiling_window)?;
  info!(
    "Found neighbor window: {} (on left: {})",
    neighbor.id(),
    neighbor_on_left
  );

  // Create a vertical split container to group them
  info!("Creating vertical split container to group windows");
  let split_container = SplitContainer::new(
    TilingDirection::Vertical,
    config.value.gaps.clone(),
  );

  // Wrap both windows together in the vertical split.
  // Order: left/top window first, right/bottom window second.
  let children = if neighbor_on_left {
    vec![neighbor.into(), tiling_window.clone().into()]
  } else {
    vec![tiling_window.clone().into(), neighbor.into()]
  };

  info!("Wrapping both windows in vertical split");
  wrap_in_split_container(
    &split_container,
    &parent,
    &children,
  )?;

  // Queue the parent workspace/container for redraw so the layout updates
  info!("Queueing containers for redraw");
  state
    .pending_sync
    .queue_containers_to_redraw(parent.tiling_children());

  info!("Toggle vertical with neighbor completed successfully");
  Ok(())
}

/// Finds the neighbor window (left first, then right).
/// Returns (neighbor, is_on_left)
fn find_neighbor_window(
  window: &TilingWindow,
) -> anyhow::Result<(TilingWindow, bool)> {
  info!("Looking for neighbor window...");
  
  // Try to find a sibling to the left first
  // prev_siblings() returns siblings in reverse order (closest first)
  let prev_tiling_siblings: Vec<_> = window
    .prev_siblings()
    .filter_map(|s| s.as_tiling_window().cloned())
    .collect();

  info!(
    "Found {} previous tiling siblings: {:?}",
    prev_tiling_siblings.len(),
    prev_tiling_siblings.iter().map(|w| w.id()).collect::<Vec<_>>()
  );

  // prev_siblings() returns in reverse order, so .first() is the immediate left neighbor
  if let Some(left_neighbor) = prev_tiling_siblings.first() {
    info!("Using immediate left neighbor: {}", left_neighbor.id());
    return Ok((left_neighbor.clone(), true));
  }

  // Otherwise, try to find a sibling to the right
  info!("No left neighbor, looking for right neighbor...");
  let next_tiling_siblings: Vec<_> = window
    .next_siblings()
    .filter_map(|s| s.as_tiling_window().cloned())
    .collect();
  
  info!(
    "Found {} next tiling siblings: {:?}",
    next_tiling_siblings.len(),
    next_tiling_siblings.iter().map(|w| w.id()).collect::<Vec<_>>()
  );

  next_tiling_siblings
    .first()
    .cloned()
    .map(|neighbor| (neighbor, false))
    .context("No neighbor window found.")
}
