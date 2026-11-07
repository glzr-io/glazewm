use std::{collections::HashMap, time::Duration};

use anyhow::Context;
use tracing::info;
use uuid::Uuid;
use wm_common::{GapsConfig, TilingDirection, WmEvent};

use super::remove_window_from_tree;
use crate::{
  commands::container::{
    attach_container, detach_container, set_focused_descendant,
  },
  models::{
    Container, LayoutNode, LayoutSnapshot, ParkedWindow, SplitContainer,
    TilingContainer, TilingWindow, WindowContainer, Workspace,
  },
  traits::{
    CommonGetters, TilingDirectionGetters, TilingSizeGetters,
    WindowGetters,
  },
  wm_state::WmState,
};

/// Maximum time between windows being parked for them to share a layout
/// snapshot.
///
/// Windows on a virtual desktop are all hidden within a short period when
/// switching desktops.
const SNAPSHOT_REUSE_WINDOW: Duration = Duration::from_secs(1);

/// Detaches a window that was moved off the current virtual desktop, so
/// that it can later be restored via [`restore_parked_window`].
///
/// For tiling windows, the workspace's layout is captured before the
/// first of its windows is parked.
///
/// Emits a `WmEvent::WindowUnmanaged` event, since the window is no longer
/// part of the container tree.
#[allow(clippy::needless_pass_by_value)]
pub fn park_window(
  window: WindowContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  info!("Parking window moved off virtual desktop: {window}");

  let workspace = window.workspace().context("No workspace.")?;

  let layout_snapshot_id = match &window {
    WindowContainer::TilingWindow(_) => {
      Some(layout_snapshot_for_park(&window, &workspace, state))
    }
    WindowContainer::NonTilingWindow(_) => None,
  };

  remove_window_from_tree(&window, state)?;

  state.emit_event(WmEvent::WindowUnmanaged {
    unmanaged_id: window.id(),
    #[allow(clippy::cast_possible_wrap, clippy::unnecessary_cast)]
    unmanaged_handle: window.native().id().0 as isize,
  });

  state.parked_windows.push(ParkedWindow {
    window,
    workspace,
    layout_snapshot_id,
  });

  Ok(())
}

/// Re-attaches a parked window to its previous workspace.
///
/// Tiling windows are restored to their previous position in the layout
/// once all tiling windows in the workspace are from the same layout
/// snapshot. Otherwise, they are appended to the workspace.
///
/// Returns the parked window back if its previous workspace no longer
/// exists, in which case the caller should manage it as a new window.
pub fn restore_parked_window(
  parked: ParkedWindow,
  state: &mut WmState,
) -> anyhow::Result<Option<ParkedWindow>> {
  // Workspace might have been deactivated while the window was parked.
  if parked.workspace.is_detached() {
    state.prune_layout_snapshots();
    return Ok(Some(parked));
  }

  let window = parked.window.clone();
  let workspace = parked.workspace.clone();

  attach_container(
    &window.clone().into(),
    &workspace.clone().into(),
    None,
  )?;

  let snapshot = parked.layout_snapshot_id.and_then(|id| {
    state
      .layout_snapshots
      .iter_mut()
      .find(|snapshot| snapshot.id == id)
  });

  let is_layout_restored = match snapshot {
    Some(snapshot) => {
      snapshot.is_restoring = true;
      apply_layout_snapshot(snapshot)?
    }
    None => false,
  };

  info!(
    "Restored window moved back to virtual desktop: {window} (layout \
     restored: {is_layout_restored})."
  );

  state.prune_layout_snapshots();

  state.emit_event(WmEvent::WindowManaged {
    managed_window: window.to_dto()?,
  });

  state
    .pending_sync
    .queue_container_to_redraw(window)
    .queue_containers_to_redraw(workspace.tiling_children())
    .queue_workspace_to_reorder(workspace);

  Ok(None)
}

/// Gets the ID of the layout snapshot to use for parking the given
/// window. Captures a new snapshot of the workspace if there is no
/// suitable existing one.
fn layout_snapshot_for_park(
  window: &WindowContainer,
  workspace: &Workspace,
  state: &mut WmState,
) -> Uuid {
  let existing = state.layout_snapshots.iter_mut().find(|snapshot| {
    snapshot.workspace.id() == workspace.id()
      && !snapshot.is_restoring
      && snapshot.last_parked_at.elapsed() < SNAPSHOT_REUSE_WINDOW
      && snapshot.window_ids().contains(&window.id())
  });

  if let Some(snapshot) = existing {
    snapshot.last_parked_at = std::time::Instant::now();
    return snapshot.id;
  }

  let snapshot = LayoutSnapshot::capture(workspace);
  let id = snapshot.id;
  state.layout_snapshots.push(snapshot);
  id
}

/// Rebuilds the tiling layout of the snapshot's workspace.
///
/// Only applied if every tiling window currently in the workspace is part
/// of the snapshot. Windows from the snapshot that are not in the
/// workspace are skipped, and redundant split containers are omitted.
///
/// Returns whether the layout was applied.
pub(crate) fn apply_layout_snapshot(
  snapshot: &LayoutSnapshot,
) -> anyhow::Result<bool> {
  let workspace = &snapshot.workspace;

  let present_windows = workspace
    .descendants()
    .filter_map(|container| container.as_tiling_window().cloned())
    .map(|window| (window.id(), window))
    .collect::<HashMap<_, _>>();

  let snapshot_ids = snapshot.window_ids();

  if present_windows.is_empty()
    || !present_windows.keys().all(|id| snapshot_ids.contains(id))
  {
    return Ok(false);
  }

  // Get the focus order of windows in the workspace, so that it can be
  // preserved after rebuilding the layout.
  let focus_order = workspace
    .descendant_focus_order()
    .filter(|container| container.as_window_container().is_ok())
    .collect::<Vec<_>>();

  let mut plans = Vec::new();
  for node in &snapshot.nodes {
    if let Some(plan) = LayoutPlan::from_node(node, &present_windows) {
      plan.push_into(&mut plans, &snapshot.tiling_direction);
    }
  }

  // A workspace with a single split container child takes on the split's
  // tiling direction instead (e.g. `H[V[1, 2]]` -> `V[1, 2]`).
  let (tiling_direction, plans) = match plans.as_slice() {
    [LayoutPlan::Split {
      tiling_direction,
      children,
      ..
    }] => (tiling_direction.clone(), children.clone()),
    _ => (snapshot.tiling_direction.clone(), plans),
  };

  for window in present_windows.values() {
    detach_container(window.clone().into())?;
  }

  workspace.set_tiling_direction(tiling_direction);
  LayoutPlan::attach_all(&plans, &workspace.clone().into())?;

  for container in focus_order.iter().rev() {
    let top_level = container
      .self_and_ancestors()
      .find(|ancestor| {
        ancestor
          .parent()
          .is_some_and(|parent| parent.id() == workspace.id())
      })
      .context("Container is not in workspace.")?;

    set_focused_descendant(container, Some(&top_level));
  }

  Ok(true)
}

/// Tiling layout to build from a `LayoutSnapshot`, limited to the windows
/// that are present.
#[derive(Debug, Clone)]
enum LayoutPlan {
  Window {
    window: TilingWindow,
    tiling_size: f32,
  },
  Split {
    tiling_direction: TilingDirection,
    gaps_config: GapsConfig,
    tiling_size: f32,
    children: Vec<LayoutPlan>,
  },
}

impl LayoutPlan {
  /// Creates a plan for the given snapshot node. Returns `None` if none of
  /// the node's windows are present.
  fn from_node(
    node: &LayoutNode,
    present_windows: &HashMap<Uuid, TilingWindow>,
  ) -> Option<Self> {
    match node {
      LayoutNode::Window { id, tiling_size } => {
        present_windows.get(id).map(|window| Self::Window {
          window: window.clone(),
          tiling_size: *tiling_size,
        })
      }
      LayoutNode::Split {
        tiling_direction,
        gaps_config,
        tiling_size,
        children,
      } => {
        let mut child_plans = Vec::new();

        for child in children {
          if let Some(plan) = Self::from_node(child, present_windows) {
            plan.push_into(&mut child_plans, tiling_direction);
          }
        }

        match child_plans.len() {
          0 => None,
          // A split container with a single child is redundant.
          1 => child_plans
            .pop()
            .map(|child| child.with_tiling_size(*tiling_size)),
          _ => Some(Self::Split {
            tiling_direction: tiling_direction.clone(),
            gaps_config: gaps_config.clone(),
            tiling_size: *tiling_size,
            children: child_plans,
          }),
        }
      }
    }
  }

  fn tiling_size(&self) -> f32 {
    match self {
      Self::Window { tiling_size, .. }
      | Self::Split { tiling_size, .. } => *tiling_size,
    }
  }

  fn with_tiling_size(self, tiling_size: f32) -> Self {
    match self {
      Self::Window { window, .. } => Self::Window {
        window,
        tiling_size,
      },
      Self::Split {
        tiling_direction,
        gaps_config,
        children,
        ..
      } => Self::Split {
        tiling_direction,
        gaps_config,
        tiling_size,
        children,
      },
    }
  }

  /// Adds the plan to a list of siblings in a parent with the given tiling
  /// direction.
  ///
  /// A split container with the same tiling direction as its parent is
  /// redundant, so its children are added in its place instead.
  fn push_into(
    self,
    siblings: &mut Vec<Self>,
    parent_direction: &TilingDirection,
  ) {
    match self {
      Self::Split {
        tiling_direction,
        tiling_size,
        children,
        ..
      } if tiling_direction == *parent_direction => {
        let total_size = total_tiling_size(&children);

        for child in children {
          let child_size = child.tiling_size() / total_size * tiling_size;
          siblings.push(child.with_tiling_size(child_size));
        }
      }
      plan => siblings.push(plan),
    }
  }

  /// Attaches the given plans as children of the parent, with tiling sizes
  /// scaled to fill the parent.
  fn attach_all(plans: &[Self], parent: &Container) -> anyhow::Result<()> {
    let total_size = total_tiling_size(plans);
    let mut attached = Vec::<(TilingContainer, f32)>::new();

    for plan in plans {
      let container: TilingContainer = match plan {
        Self::Window { window, .. } => {
          attach_container(&window.clone().into(), parent, None)?;
          window.clone().into()
        }
        Self::Split {
          tiling_direction,
          gaps_config,
          children,
          ..
        } => {
          let split = SplitContainer::new(
            tiling_direction.clone(),
            gaps_config.clone(),
          );

          attach_container(&split.clone().into(), parent, None)?;
          Self::attach_all(children, &split.clone().into())?;
          split.into()
        }
      };

      attached.push((container, plan.tiling_size() / total_size));
    }

    // Sizes are set after attaching, since attaching resizes siblings.
    for (container, tiling_size) in attached {
      container.set_tiling_size(tiling_size);
    }

    Ok(())
  }
}

/// Sum of the tiling sizes of the given plans. Defaults to 1 if the sum is
/// not positive, to avoid dividing by zero.
fn total_tiling_size(plans: &[LayoutPlan]) -> f32 {
  let total = plans.iter().map(LayoutPlan::tiling_size).sum::<f32>();

  if total > 0.0 {
    total
  } else {
    1.0
  }
}

#[cfg(test)]
mod tests {
  use wm_common::TilingDirection;

  use super::apply_layout_snapshot;
  use crate::{
    commands::container::{
      attach_container, detach_container, flatten_child_split_containers,
    },
    models::{
      Container, LayoutSnapshot, SplitContainer, TilingContainer,
      TilingWindow, Workspace,
    },
    traits::{CommonGetters, TilingDirectionGetters, TilingSizeGetters},
  };

  /// All orderings of the given indices.
  fn permutations(indices: &[usize]) -> Vec<Vec<usize>> {
    if indices.len() <= 1 {
      return vec![indices.to_vec()];
    }

    (0..indices.len())
      .flat_map(|i| {
        let mut rest = indices.to_vec();
        let first = rest.remove(i);

        permutations(&rest).into_iter().map(move |mut perm| {
          perm.insert(0, first);
          perm
        })
      })
      .collect()
  }

  /// Describes the tiling layout of a container, e.g. `H[0:0.50,
  /// V[1:0.30, 2:0.70]:0.50]`. Windows are named by their index in
  /// `windows`.
  fn describe(container: &Container, windows: &[TilingWindow]) -> String {
    let children = container
      .tiling_children()
      .map(|child| {
        let label = match &child {
          TilingContainer::TilingWindow(window) => windows
            .iter()
            .position(|w| w.id() == window.id())
            .unwrap()
            .to_string(),
          TilingContainer::Split(split) => {
            describe(&split.clone().into(), windows)
          }
        };

        format!("{label}:{:.2}", child.tiling_size())
      })
      .collect::<Vec<_>>()
      .join(", ");

    let direction = match container.as_direction_container() {
      Ok(container) => match container.tiling_direction() {
        TilingDirection::Horizontal => "H",
        TilingDirection::Vertical => "V",
      },
      Err(_) => "?",
    };

    format!("{direction}[{children}]")
  }

  /// Creates the layout `H[0, V[1, H[2, 3]]]` with uneven sizes.
  fn nested_workspace() -> (Workspace, Vec<TilingWindow>) {
    let windows = (0..4)
      .map(|_| TilingWindow::mock().call())
      .collect::<Vec<_>>();

    let inner = SplitContainer::mock()
      .tiling_direction(TilingDirection::Horizontal)
      .tiling_containers(vec![
        windows[2].clone().into(),
        windows[3].clone().into(),
      ])
      .call();

    let middle = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![
        windows[1].clone().into(),
        inner.clone().into(),
      ])
      .call();

    let workspace = Workspace::mock()
      .tiling_containers(vec![
        windows[0].clone().into(),
        middle.clone().into(),
      ])
      .call();

    windows[0].set_tiling_size(0.6);
    middle.set_tiling_size(0.4);
    windows[1].set_tiling_size(0.25);
    inner.set_tiling_size(0.75);
    windows[2].set_tiling_size(0.3);
    windows[3].set_tiling_size(0.7);

    (workspace, windows)
  }

  /// Detaches a window the same way as `remove_window_from_tree`.
  fn detach_window(window: &TilingWindow) {
    let ancestors = window.ancestors().take(3).collect::<Vec<_>>();
    detach_container(window.clone().into()).unwrap();

    for ancestor in ancestors.iter().rev() {
      flatten_child_split_containers(ancestor).unwrap();
    }
  }

  #[test]
  fn restores_nested_layout_for_any_park_and_restore_order() {
    let expected = "H[0:0.60, V[1:0.25, H[2:0.30, 3:0.70]:0.75]:0.40]";

    for park_order in permutations(&[0, 1, 2, 3]) {
      for restore_order in permutations(&[0, 1, 2, 3]) {
        let (workspace, windows) = nested_workspace();
        let workspace_container: Container = workspace.clone().into();
        assert_eq!(describe(&workspace_container, &windows), expected);

        let snapshot = LayoutSnapshot::capture(&workspace);

        for &i in &park_order {
          detach_window(&windows[i]);
        }

        assert!(!workspace.has_children());

        for &i in &restore_order {
          attach_container(
            &windows[i].clone().into(),
            &workspace_container,
            None,
          )
          .unwrap();

          assert!(apply_layout_snapshot(&snapshot).unwrap());
        }

        assert_eq!(
          describe(&workspace_container, &windows),
          expected,
          "park {park_order:?}, restore {restore_order:?}"
        );
      }
    }
  }

  #[test]
  fn partially_restored_layout_omits_redundant_splits() {
    let (workspace, windows) = nested_workspace();
    let workspace_container: Container = workspace.clone().into();
    let snapshot = LayoutSnapshot::capture(&workspace);

    for window in &windows {
      detach_window(window);
    }

    // Only windows 1 and 2 are restored, so the inner splits collapse into
    // a single vertical layout.
    for i in [2, 1] {
      attach_container(
        &windows[i].clone().into(),
        &workspace_container,
        None,
      )
      .unwrap();
      assert!(apply_layout_snapshot(&snapshot).unwrap());
    }

    assert_eq!(
      describe(&workspace_container, &windows),
      "V[1:0.25, 2:0.75]"
    );
  }

  #[test]
  fn skips_layout_when_workspace_has_other_windows() {
    let (workspace, windows) = nested_workspace();
    let workspace_container: Container = workspace.clone().into();
    let snapshot = LayoutSnapshot::capture(&workspace);

    detach_window(&windows[0]);

    let other_window = TilingWindow::mock().call();
    attach_container(&other_window.into(), &workspace_container, None)
      .unwrap();

    attach_container(
      &windows[0].clone().into(),
      &workspace_container,
      None,
    )
    .unwrap();

    assert!(!apply_layout_snapshot(&snapshot).unwrap());
  }
}
