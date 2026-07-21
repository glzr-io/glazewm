//! The `columns` feature: declarative, centered-focus column layouts.
//!
//! Layout is split across three concerns:
//! - [`spec`]: pure parsing of a column `spec` and distribution of windows
//!   into columns (no tree dependency, unit-tested).
//! - [`grid`]: the [`ColumnGrid`] bridge that lifts the container tree
//!   into a flat grid and renders it back.
//! - this module: the commands and config resolution that drive the two.

mod grid;
mod spec;

use anyhow::Context;
use uuid::Uuid;
use wm_common::{ColumnBias, ColumnLayout};
use wm_platform::{Direction, Rect};

use self::{
  grid::ColumnGrid,
  spec::{
    column_widths, distribute_columns, parse_columns_spec, ColumnKind,
  },
};
use crate::{
  commands::{
    container::focus_container_by_id,
    window::move_to_workspace_in_direction,
  },
  models::{Container, TilingWindow, WindowContainer, Workspace},
  traits::{CommonGetters, PositionGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Arranges the focused workspace into a centered-focus layout described
/// by a comma-separated column `spec`, laid out left-to-right. Each token
/// is one column: a number is that many windows stacked, `*` claims an
/// even share of the leftover windows, and `C` is the wide center (the
/// focused window; at most one). E.g. `*,C,*` is a center flanked by two
/// even stacks, `C,*` drops the left band for a narrow monitor, and
/// `2,1,C,3` is fully explicit. `C` is optional: a spec without one (e.g.
/// `*,*` or `2,2`) lays the windows into plain equal-width columns with no
/// wide center — `center` is ignored and no window is privileged.
///
/// The center window fills the `C` column at `center` width (a fraction of
/// the workspace, clamped to `0.1..=0.9`); the remaining columns split the
/// rest of the width evenly. Windows are assigned in on-screen order, left
/// to right. When `*` columns can't split the leftovers evenly, `bias`
/// decides which end claims the odd window(s). Any window still unplaced
/// (fixed counts under-specify the total and there is no `*`) is appended
/// to the last non-center column. Columns that end up empty are dropped
/// and the widths renormalise, so a wide spec still degrades cleanly on a
/// small monitor.
///
/// The `C` column takes `preferred_center` when set and that window is
/// still present, otherwise the focused window, otherwise the middle
/// window by position. Passing the current center as `preferred_center`
/// keeps the layout stable across reapplies that aren't meant to move the
/// center (e.g. after a side window closes).
pub fn apply_columns(
  workspace: &Workspace,
  spec: &str,
  center: f32,
  bias: &ColumnBias,
  preferred_center: Option<Uuid>,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let kinds = parse_columns_spec(spec)?;

  let mut windows = workspace
    .descendants()
    .filter_map(|container| match container {
      Container::TilingWindow(window) => Some(window),
      _ => None,
    })
    .collect::<Vec<_>>();

  if windows.len() < 2 {
    return Ok(());
  }

  windows.sort_by_key(|window| {
    window.to_rect().map_or((0, 0), |rect| (rect.x(), rect.y()))
  });

  let columns = if kinds.iter().any(|k| matches!(k, ColumnKind::Center)) {
    // Center = the preferred center window if it's still present, else the
    // focused tiling window, else the middle one by position.
    let focused_id = focused_window_id(workspace);

    let center_index = preferred_center
      .and_then(|id| windows.iter().position(|window| window.id() == id))
      .or_else(|| {
        focused_id.and_then(|id| {
          windows.iter().position(|window| window.id() == id)
        })
      })
      .unwrap_or(windows.len() / 2);

    let center_window = windows[center_index].clone();

    // A different window is taking the center, so the one it phases out
    // becomes the `center` toggle's swap-back target.
    remember_outgoing_center(workspace, center_window.id(), state);

    let rest = windows
      .iter()
      .enumerate()
      .filter(|(index, _)| *index != center_index)
      .map(|(_, window)| window.clone())
      .collect::<Vec<_>>();

    distribute_columns(&kinds, Some(center_window), rest, bias)
  } else {
    // No `C`: every window flows into the equal-width columns in on-screen
    // order and none is pulled out as a privileged center.
    distribute_columns(&kinds, None, windows, bias)
  };

  let widths = column_widths(&kinds, center.clamp(0.1, 0.9));

  ColumnGrid { columns, widths }.render(workspace, state, config)
}

/// Assigns a column layout to the workspace and applies it immediately.
/// The assignment is stored on the workspace and reapplied whenever the
/// workspace is focused (see `focus_workspace`), until it is unassigned or
/// the config is reloaded. Runtime assignments are ephemeral; edit
/// `config.yaml` to persist.
pub fn assign_columns(
  workspace: &Workspace,
  spec: &str,
  center: f32,
  bias: &ColumnBias,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let mut workspace_config = workspace.config();
  workspace_config.columns = Some(ColumnLayout {
    spec: spec.to_string(),
    center,
    bias: bias.clone(),
  });
  workspace.set_config(workspace_config);

  apply_columns(workspace, spec, center, bias, None, state, config)
}

/// Clears any column layout assigned to the workspace, so switching to it
/// no longer reapplies a layout. The current arrangement is left
/// untouched.
pub fn unassign_columns(workspace: &Workspace) {
  let mut workspace_config = workspace.config();
  workspace_config.columns = None;
  workspace.set_config(workspace_config);
}

/// Resolves the columns a workspace should currently use: its own assigned
/// `columns` if set, otherwise the first `general.default_columns` rule
/// whose aspect-ratio band contains the workspace's current monitor's
/// aspect ratio (`width / height`).
///
/// Returns `None` when the workspace has no assignment and matches no
/// default rule (or matches an explicit `default`/`none` rule), in which
/// case the workspace keeps the default tiling. Re-resolved on every
/// reapply, so a workspace moved to a differently-shaped monitor picks up
/// that monitor's default.
pub fn effective_columns(
  workspace: &Workspace,
  config: &UserConfig,
) -> anyhow::Result<Option<ColumnLayout>> {
  if let Some(columns) = workspace.config().columns {
    return Ok(Some(columns));
  }

  let Some(default_columns) = &config.value.general.default_columns else {
    return Ok(None);
  };

  let monitor_rect =
    workspace.monitor().context("No monitor.")?.to_rect()?;
  #[allow(clippy::cast_precision_loss)]
  let aspect_ratio =
    monitor_rect.width() as f32 / monitor_rect.height() as f32;

  Ok(default_columns.columns_for(aspect_ratio))
}

/// Reapplies the workspace's effective columns, if any (see
/// [`effective_columns`]: its own assignment, else a monitor-shape-derived
/// `general.default_columns` rule). Called when a workspace gains focus so
/// its layout re-asserts on switch-in. A no-op when nothing resolves or
/// the workspace has fewer than two tiling windows.
///
/// `preferred_center` pins which window lands in the `C` column; pass
/// `None` to center the focused window (switch-in, new window), or the
/// current center (see [`workspace_center_window_id`]) to keep it stable
/// across a reapply that shouldn't move the center, such as after a side
/// window closes.
///
/// The `C` column is sized from the columns' stored `center`, which a
/// manual resize updates at runtime (see [`store_center_width`]). Reading
/// from the stored template keeps the center width stable across reapplies
/// (window add/remove, switch-in) instead of drifting with the tree's
/// transient sizes; config reload restores the spec value by replacing the
/// config.
///
/// Returns whether a layout was reapplied (i.e. the workspace has
/// effective columns), so callers can follow up — e.g. resetting window
/// effects.
pub fn reapply_assigned_columns(
  workspace: &Workspace,
  preferred_center: Option<Uuid>,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<bool> {
  let Some(columns) = effective_columns(workspace, config)? else {
    return Ok(false);
  };

  apply_columns(
    workspace,
    &columns.spec,
    columns.center,
    &columns.bias,
    preferred_center,
    state,
    config,
  )?;

  Ok(true)
}

/// Reapplies assigned columns after a tiling window moves from `source` to
/// `target`. The source re-tidies around `source_center` — its center
/// before the move, captured by the caller so closing the gap doesn't
/// shift it — and the moved window becomes the center of the target's
/// layout. A no-op for either workspace that has no assigned columns.
pub fn reapply_columns_after_move(
  source: &Workspace,
  source_center: Option<Uuid>,
  target: &Workspace,
  moved_window_id: Uuid,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  reapply_assigned_columns(source, source_center, state, config)?;
  reapply_assigned_columns(target, Some(moved_window_id), state, config)?;
  Ok(())
}

/// Id of the window currently in the workspace's center — the top of its
/// widest column — if the workspace has any tiling windows. Capture this
/// before unmanaging a closing window so a follow-up
/// `reapply_assigned_columns` can keep that window centered.
pub fn workspace_center_window_id(workspace: &Workspace) -> Option<Uuid> {
  ColumnGrid::read(workspace).center_window_id()
}

/// On-screen rect of the workspace's wide center window — the top of its
/// widest column — or `None` when the layout has no center (equal-width
/// columns, or fewer than two windows). Capture this before a drag drops a
/// window back into tiling so the follow-up reapply can tell whether the
/// drop landed on the center band and should take it over.
pub fn workspace_center_rect(workspace: &Workspace) -> Option<Rect> {
  let grid = ColumnGrid::read(workspace);
  if !grid.has_center() {
    return None;
  }

  let center = grid.center_index();
  grid.columns.get(center)?.first()?.to_rect().ok()
}

/// Remembers the window leaving the center as the `center` command's
/// swap-back target.
///
/// Given the `new_center` a layout is about to install, this records the
/// window currently in the center (when different) into
/// `state.last_centered_out`, so pressing `center` on the incoming window
/// toggles back to the one it displaced — e.g. after a freshly opened
/// window auto-centers.
///
/// Must be called while the outgoing center is still in place, before the
/// workspace is rebuilt. A no-op when the center is unchanged or the
/// workspace has no center yet.
fn remember_outgoing_center(
  workspace: &Workspace,
  new_center: Uuid,
  state: &mut WmState,
) {
  if let Some(previous) = workspace_center_window_id(workspace) {
    if previous != new_center {
      state.last_centered_out = Some(previous);
    }
  }
}

/// Current width fraction of the workspace's center column — its widest
/// column — when it has a laid-out columns of at least two tiling windows.
///
/// Returns `None` for a workspace with fewer than two tiling windows,
/// where no meaningful center width exists.
fn workspace_center_width(workspace: &Workspace) -> Option<f32> {
  let grid = ColumnGrid::read(workspace);

  if grid.window_count() < 2 {
    return None;
  }

  grid.center_width()
}

/// Records the workspace's current center-column width into its assigned
/// columns, so a manual resize of the center becomes the template width
/// used by later reapplies (window add/remove, switch-in). Call this after
/// a user resize; the runtime value lives on the workspace config and is
/// reset to the spec by config reload or restart.
///
/// A no-op when the workspace has no assigned columns or no laid-out
/// center column (fewer than two tiling windows).
pub fn store_center_width(workspace: &Workspace) {
  let mut config = workspace.config();
  let Some(columns) = config.columns.as_mut() else {
    return;
  };

  let Some(width) = workspace_center_width(workspace) else {
    return;
  };

  columns.center = width;
  workspace.set_config(config);
}

/// Id of the focused tiling window on the workspace, if any.
fn focused_window_id(workspace: &Workspace) -> Option<Uuid> {
  workspace.descendant_focus_order().find_map(
    |container| match container {
      Container::TilingWindow(window) => Some(window.id()),
      _ => None,
    },
  )
}

/// Rotates the windows of the focused workspace by one slot, keeping the
/// existing column layout (column count, per-column window counts, and
/// column widths) fixed — only the window occupying each slot changes.
///
/// Windows travel a clockwise loop around the center column: up the left
/// columns, across through the center, down the right columns, then
/// wrapping from the bottom-right slot back to the bottom-left. So for a
/// `*,C,*` layout with 2/1/2 windows the cycle is bottom-left → top-left →
/// center → top-right → bottom-right → back to bottom-left. Clockwise is
/// the default; `ccw` reverses it. The focused slot stays focused, so its
/// occupant changes under a steady highlight and repeated rotates cycle
/// every window through it.
pub fn apply_rotate(
  workspace: &Workspace,
  ccw: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let grid = ColumnGrid::read(workspace);

  if grid.window_count() < 2 {
    return Ok(());
  }

  // Windows loop clockwise around the center column: the columns left of
  // center are traversed bottom-to-top (so `ring_slots` lists them
  // reversed), then the center and right columns top-to-bottom.
  let center = grid.center_index();

  let mut ring_slots: Vec<(usize, usize)> = Vec::new();
  for (col, column) in grid.columns.iter().enumerate().take(center) {
    ring_slots.extend((0..column.len()).map(|row| (col, row)));
  }
  let left_len = ring_slots.len();
  ring_slots[..left_len].reverse();
  for (col, column) in grid.columns.iter().enumerate().skip(center) {
    ring_slots.extend((0..column.len()).map(|row| (col, row)));
  }

  // The window currently in each ring slot, in ring order.
  let ring: Vec<TilingWindow> = ring_slots
    .iter()
    .map(|&(c, r)| grid.columns[c][r].clone())
    .collect();

  // Remember which ring position holds focus so the same physical slot
  // stays focused after the windows rotate beneath it.
  let focused_pos = focused_window_id(workspace)
    .and_then(|id| ring.iter().position(|window| window.id() == id));

  // Clockwise: every window advances one slot, so slot `i` takes slot
  // `i-1`'s former occupant (last wraps to first). Counter-clockwise is
  // the reverse.
  let mut rotated = ring;
  if ccw {
    rotated.rotate_left(1);
  } else {
    rotated.rotate_right(1);
  }

  let focus_target = focused_pos.map(|pos| rotated[pos].id());

  // Place each rotated window back into its ring slot, preserving the
  // exact column shape.
  let mut columns: Vec<Vec<Option<TilingWindow>>> = grid
    .columns
    .iter()
    .map(|col| vec![None; col.len()])
    .collect();
  for (&(c, r), window) in ring_slots.iter().zip(rotated) {
    columns[c][r] = Some(window);
  }
  let columns = columns
    .into_iter()
    .map(|col| col.into_iter().flatten().collect())
    .collect::<Vec<Vec<TilingWindow>>>();

  ColumnGrid {
    columns,
    widths: grid.widths,
  }
  .render(workspace, state, config)?;

  if let Some(id) = focus_target {
    focus_container_by_id(&id, state)?;
  }

  Ok(())
}

/// Swaps a window into the center slot — the top of the widest column.
///
/// When the focused window is *not* the center, it swaps into the center
/// and the old center takes its place; focus follows into the center. When
/// the focused window *is* the center, it swaps back with the window most
/// recently phased out of the center (`state.last_centered_out`), so
/// pressing `center` repeatedly toggles between two windows. That target
/// is maintained wherever the center changes — including a freshly opened
/// window auto-centering (see `remember_outgoing_center`) — so the toggle
/// can return to the previous window even when it wasn't this command that
/// centered the current one. The window leaving the center is remembered
/// as the next toggle target, and whichever window lands in the center
/// gains focus. A no-op if there is no toggle target, it no longer exists,
/// or there is nothing to swap.
pub fn apply_center(
  workspace: &Workspace,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let mut grid = ColumnGrid::read(workspace);

  if grid.window_count() < 2 {
    return Ok(());
  }

  // Equal-width columns (a `C`-less layout) have no wide center to swap
  // into, so `center` is a no-op there.
  if !grid.has_center() {
    return Ok(());
  }

  let center = grid.center_index();
  if grid.columns[center].is_empty() {
    return Ok(());
  }

  let Some(focused_id) = focused_window_id(workspace) else {
    return Ok(());
  };

  let center_id = grid.columns[center][0].id();

  // Pick the slot to swap with the center. Normally that's the focused
  // window; if the focused window already is the center, fall back to the
  // last window moved out of the center so `center` toggles between two.
  let partner = if focused_id == center_id {
    state.last_centered_out.and_then(|id| grid.find(id))
  } else {
    grid.find(focused_id)
  };

  let Some((pc, pr)) = partner else {
    return Ok(());
  };

  // Nothing to do if the partner already occupies the center.
  if (pc, pr) == (center, 0) {
    return Ok(());
  }

  let leaving = grid.columns[center][0].clone();
  let entering = grid.columns[pc][pr].clone();
  grid.columns[center][0] = entering.clone();
  grid.columns[pc][pr] = leaving.clone();

  grid.render(workspace, state, config)?;

  // Remember the window pushed out of the center as the next toggle
  // target, and move focus with the window that landed in the center.
  state.last_centered_out = Some(leaving.id());
  focus_container_by_id(&entering.id(), state)?;

  Ok(())
}

/// Moves the focused window one slot within its workspace's assigned
/// columns grid, returning whether the move was handled here.
///
/// The workspace is treated as columns left-to-right, each a top-to-bottom
/// stack of windows. Interior moves swap the focused window with a
/// neighbouring slot: `Up`/`Down` with the window above/below it in its
/// column, `Left`/`Right` with the nearest window (by row) in the adjacent
/// column. Swapping keeps every column's window count fixed, so the center
/// column — a single-window slot — always stays exactly one window: moving
/// a side window into the center displaces the old center out to the
/// vacated side slot, and moving the center window out promotes the side
/// window it swaps with into the center. The result is rendered directly
/// through `ColumnGrid::render`, so the declarative columns stay intact
/// and the spec is not re-derived from geometry, and focus follows the
/// moved window.
///
/// At a column's edge the window leaves the workspace for the monitor
/// stacked in that direction: `Up`/`Down` past the top/bottom of a column
/// and `Left`/`Right` past the outermost column both move the window to
/// the adjacent monitor's displayed workspace (a no-op when there is no
/// monitor there), re-tidying the columns left behind. This is handled
/// here rather than by the default mover, which would either reparent the
/// window into a new perpendicular split or re-column a window that has
/// stacked neighbours inside this workspace — both of which break the
/// declarative columns.
///
/// Returns `false` — leaving the caller to fall back to the default
/// directional mover — only when the workspace has no effective columns
/// (see [`effective_columns`]: its own assignment, else a monitor-shape
/// `general.default_columns` rule) or the subject is not a tiling window
/// in the grid.
pub fn move_window_in_columns(
  window: &WindowContainer,
  direction: &Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<bool> {
  let WindowContainer::TilingWindow(tiling) = window else {
    return Ok(false);
  };

  let Some(workspace) = tiling.workspace() else {
    return Ok(false);
  };

  if effective_columns(&workspace, config)?.is_none() {
    return Ok(false);
  }

  let mut grid = ColumnGrid::read(&workspace);
  let Some((col, row)) = grid.find(tiling.id()) else {
    return Ok(false);
  };

  match direction {
    Direction::Up | Direction::Down => {
      let target_row = match direction {
        Direction::Up if row > 0 => row - 1,
        Direction::Down if row + 1 < grid.columns[col].len() => row + 1,
        // Top/bottom of the column: leave for the workspace of the
        // monitor stacked in this direction (a no-op when there is none),
        // re-tidying the columns we leave behind. As with the left/right
        // edge, this is handled here rather than by the default mover,
        // which would reparent the window into a new perpendicular split
        // and break the declarative columns.
        _ => {
          move_to_workspace_in_direction(
            window, direction, state, config,
          )?;
          return Ok(true);
        }
      };
      grid.columns[col].swap(row, target_row);
    }
    Direction::Left | Direction::Right => {
      let target_col = match direction {
        Direction::Left if col > 0 => col - 1,
        Direction::Right if col + 1 < grid.columns.len() => col + 1,
        // Outermost column: leave for the adjacent monitor's workspace in
        // this direction (a no-op when there is none), which re-tidies the
        // columns we left behind. Handled here rather than by the default
        // mover, which would re-column the window inside this workspace
        // when it has stacked neighbours in its column.
        _ => {
          move_to_workspace_in_direction(
            window, direction, state, config,
          )?;
          return Ok(true);
        }
      };
      // Swap with the nearest window in the target column so both columns
      // keep their window counts — the center stays a single window.
      let target_row =
        row.min(grid.columns[target_col].len().saturating_sub(1));
      let moved = grid.columns[col][row].clone();
      grid.columns[col][row] =
        grid.columns[target_col][target_row].clone();
      grid.columns[target_col][target_row] = moved;
    }
  }

  grid.render(&workspace, state, config)?;
  focus_container_by_id(&tiling.id(), state)?;

  Ok(true)
}

#[cfg(test)]
mod tests {
  use uuid::Uuid;
  use wm_common::{ColumnBias, ParsedConfig};
  use wm_platform::{Direction, Rect};

  use super::{
    apply_center, apply_columns, apply_rotate, assign_columns,
    effective_columns, focused_window_id, grid::ColumnGrid,
    move_window_in_columns, reapply_assigned_columns, store_center_width,
    unassign_columns, workspace_center_window_id,
  };
  use crate::{
    commands::{
      container::{attach_container, focus_container_by_id},
      workspace::move_workspace_in_direction,
    },
    models::{
      Monitor, TilingContainer, TilingWindow, WindowContainer, Workspace,
    },
    test_utils::{mock_user_config, mock_wm_state},
    traits::{CommonGetters, TilingSizeGetters},
    user_config::UserConfig,
    wm_state::WmState,
  };

  /// A workspace of `window_count` tiling windows, live under a fresh
  /// `WmState`'s root so id lookups and window rects resolve. Returns the
  /// state, workspace, and the windows (in attach order).
  fn setup(
    window_count: usize,
  ) -> (WmState, Workspace, Vec<TilingWindow>) {
    let state = mock_wm_state();

    let windows = (0..window_count)
      .map(|_| TilingWindow::mock().call())
      .collect::<Vec<_>>();

    let workspace = Workspace::mock()
      .tiling_containers(
        windows
          .iter()
          .cloned()
          .map(TilingContainer::TilingWindow)
          .collect(),
      )
      .call();

    let monitor =
      Monitor::mock().workspaces(vec![workspace.clone()]).call();
    attach_container(
      &monitor.into(),
      &state.root_container.clone().into(),
      None,
    )
    .unwrap();

    (state, workspace, windows)
  }

  /// Attaches a monitor at `bounds` holding one workspace of `count`
  /// tiling windows under `state`'s root. Returns the workspace and its
  /// windows. Use distinct, non-overlapping `bounds` to place monitors
  /// adjacently so `monitor_in_direction` resolves.
  fn add_monitor(
    state: &WmState,
    bounds: Rect,
    count: usize,
  ) -> (Workspace, Vec<TilingWindow>) {
    let windows = (0..count)
      .map(|_| TilingWindow::mock().call())
      .collect::<Vec<_>>();

    let workspace = Workspace::mock()
      .tiling_containers(
        windows
          .iter()
          .cloned()
          .map(TilingContainer::TilingWindow)
          .collect(),
      )
      .call();

    let monitor = Monitor::mock()
      .bounds(bounds)
      .workspaces(vec![workspace.clone()])
      .call();

    attach_container(
      &monitor.into(),
      &state.root_container.clone().into(),
      None,
    )
    .unwrap();

    (workspace, windows)
  }

  /// A `UserConfig` whose `general.default_columns` maps wide monitors to
  /// the given `spec` (the mock monitor's aspect ratio is ~1.6).
  fn config_with_default_columns(spec: &str) -> UserConfig {
    let yaml = format!(
      "general:\n  default_columns:\n    - min_aspect_ratio: 1.5\n      spec: '{spec}'\n"
    );
    let value: ParsedConfig = serde_yaml::from_str(&yaml).unwrap();
    UserConfig::mock(value)
  }

  /// The window ids in each column, top-to-bottom, left-to-right.
  fn id_grid(workspace: &Workspace) -> Vec<Vec<Uuid>> {
    ColumnGrid::read(workspace)
      .columns
      .iter()
      .map(|column| column.iter().map(CommonGetters::id).collect())
      .collect()
  }

  /// The focused window that becomes the center keeps focus across the
  /// layout rebuild — the columns layer never drops focus on its own.
  #[test]
  fn apply_columns_preserves_focused_window() {
    let (mut state, workspace, windows) = setup(5);
    let config = mock_user_config();
    let ids = windows.iter().map(CommonGetters::id).collect::<Vec<_>>();

    focus_container_by_id(&ids[0], &mut state).unwrap();
    apply_columns(
      &workspace,
      "*,C,*",
      0.6,
      &ColumnBias::Left,
      None,
      &mut state,
      &config,
    )
    .unwrap();

    assert_eq!(focused_window_id(&workspace), Some(ids[0]));
  }

  /// `*,C,*` lays 5 windows into 2/1/2 columns with the preferred center
  /// in the wide middle column at its configured width.
  #[test]
  fn applies_star_center_star_layout() {
    let (mut state, workspace, windows) = setup(5);
    let config = mock_user_config();
    let ids = windows.iter().map(CommonGetters::id).collect::<Vec<_>>();

    apply_columns(
      &workspace,
      "*,C,*",
      0.6,
      &ColumnBias::Left,
      Some(ids[2]),
      &mut state,
      &config,
    )
    .unwrap();

    assert_eq!(
      id_grid(&workspace),
      vec![vec![ids[0], ids[1]], vec![ids[2]], vec![ids[3], ids[4]]]
    );

    let grid = ColumnGrid::read(&workspace);
    assert_eq!(grid.center_index(), 1);
    assert!((grid.center_width().unwrap() - 0.6).abs() < 1e-3);
    assert!((grid.widths[0] - 0.2).abs() < 1e-3);
    assert!((grid.widths[2] - 0.2).abs() < 1e-3);
  }

  /// `2,C,*` fixes the left column at two windows, centers the preferred
  /// window, and lets the `*` column absorb the remaining three.
  #[test]
  fn applies_fixed_column_layout() {
    let (mut state, workspace, windows) = setup(6);
    let config = mock_user_config();
    let ids = windows.iter().map(CommonGetters::id).collect::<Vec<_>>();

    apply_columns(
      &workspace,
      "2,C,*",
      0.6,
      &ColumnBias::Left,
      Some(ids[2]),
      &mut state,
      &config,
    )
    .unwrap();

    assert_eq!(
      id_grid(&workspace),
      vec![
        vec![ids[0], ids[1]],
        vec![ids[2]],
        vec![ids[3], ids[4], ids[5]],
      ]
    );

    let grid = ColumnGrid::read(&workspace);
    assert_eq!(grid.center_index(), 1);
    assert!((grid.center_width().unwrap() - 0.6).abs() < 1e-3);
    assert!((grid.widths[0] - 0.2).abs() < 1e-3);
    assert!((grid.widths[2] - 0.2).abs() < 1e-3);
  }

  /// A spec with no `C` lays every window into equal-width columns and
  /// pulls none of them out as a wide center.
  #[test]
  fn applies_equal_width_columns_without_center() {
    let (mut state, workspace, windows) = setup(4);
    let config = mock_user_config();
    let ids = windows.iter().map(CommonGetters::id).collect::<Vec<_>>();

    apply_columns(
      &workspace,
      "*,*",
      0.6,
      &ColumnBias::Left,
      None,
      &mut state,
      &config,
    )
    .unwrap();

    assert_eq!(
      id_grid(&workspace),
      vec![vec![ids[0], ids[1]], vec![ids[2], ids[3]]]
    );

    let grid = ColumnGrid::read(&workspace);
    assert!(!grid.has_center());
    assert!((grid.widths[0] - 0.5).abs() < 1e-3);
    assert!((grid.widths[1] - 0.5).abs() < 1e-3);
  }

  /// `center` has no wide column to swap into under an equal-width layout,
  /// so it leaves the grid untouched even with a window focused.
  #[test]
  fn center_is_noop_without_center() {
    let (mut state, workspace, windows) = setup(4);
    let config = mock_user_config();
    let ids = windows.iter().map(CommonGetters::id).collect::<Vec<_>>();

    apply_columns(
      &workspace,
      "*,*",
      0.6,
      &ColumnBias::Left,
      None,
      &mut state,
      &config,
    )
    .unwrap();

    focus_container_by_id(&ids[0], &mut state).unwrap();
    let before = id_grid(&workspace);
    apply_center(&workspace, &mut state, &config).unwrap();

    assert_eq!(id_grid(&workspace), before);
  }

  /// Clockwise rotate advances every window one slot around the center
  /// while the 2/1/2 column shape stays fixed.
  #[test]
  fn rotates_windows_clockwise_keeping_shape() {
    let (mut state, workspace, windows) = setup(5);
    let config = mock_user_config();
    let ids = windows.iter().map(CommonGetters::id).collect::<Vec<_>>();

    apply_columns(
      &workspace,
      "*,C,*",
      0.6,
      &ColumnBias::Left,
      Some(ids[2]),
      &mut state,
      &config,
    )
    .unwrap();

    apply_rotate(&workspace, false, &mut state, &config).unwrap();

    // Ring bottom-left → top-left → center → top-right → bottom-right,
    // each slot taking its predecessor's occupant (last wraps to first).
    assert_eq!(
      id_grid(&workspace),
      vec![vec![ids[1], ids[4]], vec![ids[0]], vec![ids[2], ids[3]]]
    );
  }

  /// Counter-clockwise rotate is the exact reverse.
  #[test]
  fn rotates_windows_counter_clockwise() {
    let (mut state, workspace, windows) = setup(5);
    let config = mock_user_config();
    let ids = windows.iter().map(CommonGetters::id).collect::<Vec<_>>();

    apply_columns(
      &workspace,
      "*,C,*",
      0.6,
      &ColumnBias::Left,
      Some(ids[2]),
      &mut state,
      &config,
    )
    .unwrap();

    apply_rotate(&workspace, true, &mut state, &config).unwrap();

    assert_eq!(
      id_grid(&workspace),
      vec![vec![ids[2], ids[0]], vec![ids[3]], vec![ids[4], ids[1]]]
    );
  }

  /// `center` swaps the focused side window into the center (focus
  /// follows), then toggles back to the displaced window on a repeat
  /// press.
  #[test]
  fn center_swaps_focused_then_toggles_back() {
    let (mut state, workspace, windows) = setup(5);
    let config = mock_user_config();
    let ids = windows.iter().map(CommonGetters::id).collect::<Vec<_>>();

    apply_columns(
      &workspace,
      "*,C,*",
      0.6,
      &ColumnBias::Left,
      Some(ids[2]),
      &mut state,
      &config,
    )
    .unwrap();

    // Focus a side window and center it: it swaps with the old center.
    focus_container_by_id(&ids[0], &mut state).unwrap();
    apply_center(&workspace, &mut state, &config).unwrap();

    assert_eq!(
      id_grid(&workspace),
      vec![vec![ids[2], ids[1]], vec![ids[0]], vec![ids[3], ids[4]]]
    );
    assert_eq!(focused_window_id(&workspace), Some(ids[0]));
    assert_eq!(state.last_centered_out, Some(ids[2]));

    // Focused window is now the center, so `center` toggles it back with
    // the window it displaced.
    apply_center(&workspace, &mut state, &config).unwrap();

    assert_eq!(
      id_grid(&workspace),
      vec![vec![ids[0], ids[1]], vec![ids[2]], vec![ids[3], ids[4]]]
    );
    assert_eq!(focused_window_id(&workspace), Some(ids[2]));
    assert_eq!(state.last_centered_out, Some(ids[0]));
  }

  /// `Up`/`Down` swaps a window with its vertical neighbour in the column.
  #[test]
  fn moves_window_within_column() {
    let (mut state, workspace, windows) = setup(5);
    // `move_window_in_columns` only runs when the workspace has effective
    // columns; a matching `default_columns` rule provides that.
    let config = config_with_default_columns("*,C,*");
    let ids = windows.iter().map(CommonGetters::id).collect::<Vec<_>>();

    apply_columns(
      &workspace,
      "*,C,*",
      0.6,
      &ColumnBias::Left,
      Some(ids[2]),
      &mut state,
      &config,
    )
    .unwrap();

    let window = WindowContainer::TilingWindow(windows[1].clone());
    let handled =
      move_window_in_columns(&window, &Direction::Up, &mut state, &config)
        .unwrap();

    assert!(handled);
    assert_eq!(
      id_grid(&workspace),
      vec![vec![ids[1], ids[0]], vec![ids[2]], vec![ids[3], ids[4]]]
    );
  }

  /// Moving a side window into the center displaces the old center out to
  /// the vacated slot, keeping the center a single window.
  #[test]
  fn moves_into_center_keeps_it_single_window() {
    let (mut state, workspace, windows) = setup(5);
    let config = config_with_default_columns("*,C,*");
    let ids = windows.iter().map(CommonGetters::id).collect::<Vec<_>>();

    apply_columns(
      &workspace,
      "*,C,*",
      0.6,
      &ColumnBias::Left,
      Some(ids[2]),
      &mut state,
      &config,
    )
    .unwrap();

    let window = WindowContainer::TilingWindow(windows[0].clone());
    let handled = move_window_in_columns(
      &window,
      &Direction::Right,
      &mut state,
      &config,
    )
    .unwrap();

    assert!(handled);
    assert_eq!(
      id_grid(&workspace),
      vec![vec![ids[2], ids[1]], vec![ids[0]], vec![ids[3], ids[4]]]
    );
  }

  /// `move` within a fixed-column layout swaps window-for-window: `Down`
  /// reorders the fixed column's stack, and `Right` into the center keeps
  /// the fixed column at two windows and the center at one.
  #[test]
  fn moves_within_fixed_column_layout() {
    let (mut state, workspace, windows) = setup(6);
    let config = config_with_default_columns("2,C,*");
    let ids = windows.iter().map(CommonGetters::id).collect::<Vec<_>>();

    apply_columns(
      &workspace,
      "2,C,*",
      0.6,
      &ColumnBias::Left,
      Some(ids[2]),
      &mut state,
      &config,
    )
    .unwrap();

    // `Down` swaps the two windows stacked in the fixed left column.
    let top = WindowContainer::TilingWindow(windows[0].clone());
    assert!(move_window_in_columns(
      &top,
      &Direction::Down,
      &mut state,
      &config
    )
    .unwrap());
    assert_eq!(
      id_grid(&workspace),
      vec![
        vec![ids[1], ids[0]],
        vec![ids[2]],
        vec![ids[3], ids[4], ids[5]],
      ]
    );

    // `Right` moves the fixed column's bottom window into the center,
    // displacing the old center into the vacated slot. Counts are
    // preserved: the fixed column keeps two, the center keeps one.
    let bottom = WindowContainer::TilingWindow(windows[0].clone());
    assert!(move_window_in_columns(
      &bottom,
      &Direction::Right,
      &mut state,
      &config
    )
    .unwrap());
    assert_eq!(
      id_grid(&workspace),
      vec![
        vec![ids[1], ids[2]],
        vec![ids[0]],
        vec![ids[3], ids[4], ids[5]],
      ]
    );
  }

  /// Moving off the outermost column sends the window to the adjacent
  /// monitor's workspace: the source re-tidies keeping its center, and the
  /// moved window becomes the target's center. (`Left` is symmetric.)
  #[test]
  fn moves_out_to_horizontally_adjacent_monitor() {
    let mut state = mock_wm_state();
    let config = config_with_default_columns("*,C,*");

    // Two 1680x1050 monitors side by side (aspect ~1.6, so both match the
    // default-columns rule).
    let (ws_left, wins_left) =
      add_monitor(&state, Rect::from_xy(0, 0, 1680, 1050), 5);
    let (ws_right, _wins_right) =
      add_monitor(&state, Rect::from_xy(1680, 0, 1680, 1050), 2);
    let ids = wins_left.iter().map(CommonGetters::id).collect::<Vec<_>>();

    apply_columns(
      &ws_left,
      "*,C,*",
      0.6,
      &ColumnBias::Left,
      Some(ids[2]),
      &mut state,
      &config,
    )
    .unwrap();

    // `ids[3]` sits in the rightmost column; moving it `Right` leaves the
    // workspace for the monitor to the right.
    let window = WindowContainer::TilingWindow(wins_left[3].clone());
    let handled = move_window_in_columns(
      &window,
      &Direction::Right,
      &mut state,
      &config,
    )
    .unwrap();

    assert!(handled);
    assert_eq!(
      wins_left[3].workspace().map(|w| w.id()),
      Some(ws_right.id())
    );
    // Source keeps its center; target adopts the arrival as its center.
    assert_eq!(workspace_center_window_id(&ws_left), Some(ids[2]));
    assert_eq!(workspace_center_window_id(&ws_right), Some(ids[3]));
  }

  /// The same edge hand-off works vertically: moving off the bottom row
  /// sends the window to the monitor below. (`Up` is symmetric.)
  #[test]
  fn moves_out_to_vertically_adjacent_monitor() {
    let mut state = mock_wm_state();
    let config = config_with_default_columns("*,C,*");

    // Two monitors stacked top/bottom.
    let (ws_top, wins_top) =
      add_monitor(&state, Rect::from_xy(0, 0, 1680, 1050), 5);
    let (ws_bottom, _wins_bottom) =
      add_monitor(&state, Rect::from_xy(0, 1050, 1680, 1050), 2);
    let ids = wins_top.iter().map(CommonGetters::id).collect::<Vec<_>>();

    apply_columns(
      &ws_top,
      "*,C,*",
      0.6,
      &ColumnBias::Left,
      Some(ids[2]),
      &mut state,
      &config,
    )
    .unwrap();

    // `ids[1]` is the bottom of the left column; `Down` leaves for the
    // monitor below.
    let window = WindowContainer::TilingWindow(wins_top[1].clone());
    let handled = move_window_in_columns(
      &window,
      &Direction::Down,
      &mut state,
      &config,
    )
    .unwrap();

    assert!(handled);
    assert_eq!(
      wins_top[1].workspace().map(|w| w.id()),
      Some(ws_bottom.id())
    );
    assert_eq!(workspace_center_window_id(&ws_top), Some(ids[2]));
    assert_eq!(workspace_center_window_id(&ws_bottom), Some(ids[1]));
  }

  /// Moving a workspace to a differently-shaped monitor via the real
  /// `move_workspace_in_direction` command re-resolves its
  /// `general.default_columns` and reapplies that monitor's layout. This
  /// covers the command's own reparent + reapply wiring, not just the
  /// resolution in isolation.
  #[test]
  fn workspace_recolumns_when_moved_to_monitor_with_different_aspect() {
    let mut state = mock_wm_state();

    // Ultrawide (2.1+) → `*,C,*`; standard widescreen (1.5+) → `C,*`.
    let yaml = "general:\n  default_columns:\n    - min_aspect_ratio: 2.1\n      spec: '*,C,*'\n    - min_aspect_ratio: 1.5\n      spec: 'C,*'\n";
    let value: ParsedConfig = serde_yaml::from_str(yaml).unwrap();
    let config = UserConfig::mock(value);

    // Ultrawide 3440x1440 (~2.39) with the workspace under test plus a
    // second workspace, so moving the first out doesn't empty the monitor
    // (which would need a workspace config to backfill). Standard
    // 1920x1080 (~1.78) sits to its right.
    let (workspace, _wins) =
      add_monitor(&state, Rect::from_xy(0, 0, 3440, 1440), 3);
    let ultrawide = workspace.monitor().unwrap();
    let filler = Workspace::mock().name("filler".to_string()).call();
    attach_container(&filler.into(), &ultrawide.clone().into(), None)
      .unwrap();
    add_monitor(&state, Rect::from_xy(3440, 0, 1920, 1080), 0);

    // On the ultrawide, the workspace resolves and applies the 3-column
    // layout.
    reapply_assigned_columns(&workspace, None, &mut state, &config)
      .unwrap();
    assert_eq!(ColumnGrid::read(&workspace).columns.len(), 3);

    // Move it right onto the standard monitor with the real command.
    move_workspace_in_direction(
      &workspace,
      &Direction::Right,
      &mut state,
      &config,
    )
    .unwrap();

    // The command re-resolved the layout for the new monitor: a 2-column
    // `C,*`, with the moved window kept centered.
    assert_eq!(
      effective_columns(&workspace, &config)
        .unwrap()
        .map(|c| c.spec),
      Some("C,*".to_string())
    );
    assert_eq!(ColumnGrid::read(&workspace).columns.len(), 2);
  }

  /// Without effective columns the move is not handled here, so the caller
  /// falls back to the default directional mover.
  #[test]
  fn move_without_columns_is_not_handled() {
    let (mut state, _workspace, windows) = setup(3);
    let config = mock_user_config();

    let window = WindowContainer::TilingWindow(windows[0].clone());
    let handled = move_window_in_columns(
      &window,
      &Direction::Left,
      &mut state,
      &config,
    )
    .unwrap();

    assert!(!handled);
  }

  /// A workspace with no assignment inherits the monitor-shape default.
  #[test]
  fn effective_columns_uses_default_when_unassigned() {
    let (_state, workspace, _windows) = setup(2);
    let config = config_with_default_columns("C,*");

    let columns = effective_columns(&workspace, &config).unwrap();
    assert_eq!(columns.map(|c| c.spec), Some("C,*".to_string()));
  }

  /// An explicit assignment wins over the monitor-shape default.
  #[test]
  fn effective_columns_prefers_assignment() {
    let (mut state, workspace, _windows) = setup(2);
    let config = config_with_default_columns("*,C,*");

    assign_columns(
      &workspace,
      "1,C",
      0.6,
      &ColumnBias::Left,
      &mut state,
      &config,
    )
    .unwrap();

    let columns = effective_columns(&workspace, &config).unwrap();
    assert_eq!(columns.map(|c| c.spec), Some("1,C".to_string()));
  }

  /// Unassigning clears the workspace's columns so it no longer resolves
  /// an assignment.
  #[test]
  fn unassign_clears_assignment() {
    let (mut state, workspace, _windows) = setup(2);
    let config = mock_user_config();

    assign_columns(
      &workspace,
      "*,C,*",
      0.6,
      &ColumnBias::Left,
      &mut state,
      &config,
    )
    .unwrap();
    assert!(workspace.config().columns.is_some());

    unassign_columns(&workspace);
    assert!(workspace.config().columns.is_none());
  }

  /// The center window id is the top of the widest column.
  #[test]
  fn reports_center_window_id() {
    let (mut state, workspace, windows) = setup(5);
    let config = mock_user_config();
    let ids = windows.iter().map(CommonGetters::id).collect::<Vec<_>>();

    apply_columns(
      &workspace,
      "*,C,*",
      0.6,
      &ColumnBias::Left,
      Some(ids[2]),
      &mut state,
      &config,
    )
    .unwrap();

    assert_eq!(workspace_center_window_id(&workspace), Some(ids[2]));
  }

  /// A manual resize of the center is captured into the assignment so
  /// later reapplies use the new width.
  #[test]
  fn store_center_width_records_resize() {
    let (mut state, workspace, windows) = setup(5);
    let config = mock_user_config();

    assign_columns(
      &workspace,
      "*,C,*",
      0.6,
      &ColumnBias::Left,
      &mut state,
      &config,
    )
    .unwrap();

    // Simulate the user widening the center column (a single-window
    // column, so its window's tiling size is the column width).
    let center_id = workspace_center_window_id(&workspace).unwrap();
    let center = windows
      .iter()
      .find(|window| window.id() == center_id)
      .unwrap();
    center.set_tiling_size(0.7);
    store_center_width(&workspace);

    let stored = workspace.config().columns.unwrap().center;
    assert!((stored - 0.7).abs() < 1e-3);
  }
}
