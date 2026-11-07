use std::time::Instant;

use uuid::Uuid;
use wm_common::{GapsConfig, TilingDirection};

use super::{TilingContainer, WindowContainer, Workspace};
use crate::traits::{
  CommonGetters, TilingDirectionGetters, TilingSizeGetters,
};

/// A managed window that was moved off the current virtual desktop.
///
/// The window is detached from the container tree while it is off the
/// current virtual desktop, and is restored once its virtual desktop
/// becomes active again.
#[derive(Debug, Clone)]
pub struct ParkedWindow {
  /// The detached window container.
  pub window: WindowContainer,

  /// Workspace that the window belonged to before it was parked.
  pub workspace: Workspace,

  /// ID of the `LayoutSnapshot` to restore the window's position from.
  /// Only applicable to tiling windows.
  pub layout_snapshot_id: Option<Uuid>,
}

/// Tiling layout of a workspace, captured before its windows were moved
/// off the current virtual desktop.
///
/// Windows on a virtual desktop are hidden one at a time when switching
/// desktops. Detaching them one by one causes split containers to be
/// flattened, so the layout is captured up front and rebuilt as the
/// windows are restored.
#[derive(Debug, Clone)]
pub struct LayoutSnapshot {
  pub id: Uuid,

  /// Workspace that the layout belongs to.
  pub workspace: Workspace,

  /// Tiling direction of the workspace at the time of capture.
  pub tiling_direction: TilingDirection,

  /// Tiling children of the workspace at the time of capture.
  pub nodes: Vec<LayoutNode>,

  /// When a window was last parked using this snapshot.
  pub last_parked_at: Instant,

  /// Whether any window has been restored from this snapshot. Windows
  /// are no longer added to a snapshot once restoring has begun.
  pub is_restoring: bool,
}

/// A tiling container within a `LayoutSnapshot`.
#[derive(Debug, Clone)]
pub enum LayoutNode {
  Window {
    id: Uuid,
    tiling_size: f32,
  },
  Split {
    tiling_direction: TilingDirection,
    gaps_config: GapsConfig,
    tiling_size: f32,
    children: Vec<LayoutNode>,
  },
}

impl LayoutSnapshot {
  /// Captures the current tiling layout of the given workspace.
  #[must_use]
  pub fn capture(workspace: &Workspace) -> Self {
    Self {
      id: Uuid::new_v4(),
      workspace: workspace.clone(),
      tiling_direction: workspace.tiling_direction(),
      nodes: workspace
        .tiling_children()
        .map(|child| LayoutNode::capture(&child))
        .collect(),
      last_parked_at: Instant::now(),
      is_restoring: false,
    }
  }

  /// IDs of all windows in the snapshot.
  #[must_use]
  pub fn window_ids(&self) -> Vec<Uuid> {
    let mut ids = Vec::new();

    for node in &self.nodes {
      node.collect_window_ids(&mut ids);
    }

    ids
  }
}

impl LayoutNode {
  /// Captures the layout of the given tiling container and its
  /// descendants.
  #[must_use]
  pub fn capture(container: &TilingContainer) -> Self {
    match container {
      TilingContainer::TilingWindow(window) => Self::Window {
        id: window.id(),
        tiling_size: window.tiling_size(),
      },
      TilingContainer::Split(split) => Self::Split {
        tiling_direction: split.tiling_direction(),
        gaps_config: split.gaps_config().clone(),
        tiling_size: split.tiling_size(),
        children: split
          .tiling_children()
          .map(|child| Self::capture(&child))
          .collect(),
      },
    }
  }

  fn collect_window_ids(&self, ids: &mut Vec<Uuid>) {
    match self {
      Self::Window { id, .. } => ids.push(*id),
      Self::Split { children, .. } => {
        for child in children {
          child.collect_window_ids(ids);
        }
      }
    }
  }
}
