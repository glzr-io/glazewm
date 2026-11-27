//! The `ColumnGrid`: the single bridge between the window container tree
//! and the flat column grid the column commands manipulate.

use uuid::Uuid;
use wm_common::TilingDirection;

use crate::{
  commands::container::{
    move_container_within_tree, wrap_in_split_container,
  },
  models::{
    Container, SplitContainer, TilingContainer, TilingWindow, Workspace,
  },
  traits::{CommonGetters, TilingDirectionGetters, TilingSizeGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// A workspace's tiling windows viewed as a left-to-right sequence of
/// columns, each a top-to-bottom stack of windows, paired with each
/// column's current width fraction.
///
/// This is the one place that knows how the declarative columns map onto
/// the container tree: [`ColumnGrid::read`] lifts the tree into the grid
/// and [`ColumnGrid::render`] rebuilds the tree from it. The commands
/// operate purely on the grid in between. The layout's center is the
/// column holding the workspace's recorded center window (see
/// [`ColumnGrid::read`]).
pub(super) struct ColumnGrid {
  /// Columns left-to-right, each a top-to-bottom stack of windows.
  pub columns: Vec<Vec<TilingWindow>>,

  /// Width fraction of each column, index-aligned with `columns`.
  pub widths: Vec<f32>,

  /// Index of the `C` column, or `None` for a layout without a center.
  pub center: Option<usize>,
}

impl ColumnGrid {
  /// Reads the workspace's top-level columns — each a top-to-bottom list
  /// of windows — paired with their current widths.
  ///
  /// The center is the column holding the window recorded by the last
  /// render. Without a recorded window still in the workspace, it falls
  /// back to a single strictly-widest column, if any.
  pub fn read(workspace: &Workspace) -> Self {
    let mut columns: Vec<Vec<TilingWindow>> = Vec::new();
    let mut widths: Vec<f32> = Vec::new();

    for child in workspace.tiling_children() {
      match child {
        TilingContainer::TilingWindow(window) => {
          widths.push(window.tiling_size());
          columns.push(vec![window]);
        }
        TilingContainer::Split(split) => {
          widths.push(split.tiling_size());
          columns.push(
            split
              .descendants()
              .filter_map(|container| match container {
                Container::TilingWindow(window) => Some(window),
                _ => None,
              })
              .collect(),
          );
        }
      }
    }

    let recorded = workspace.center_window_id().and_then(|id| {
      columns
        .iter()
        .position(|column| column.iter().any(|window| window.id() == id))
    });
    let center = recorded.or_else(|| strictly_widest(&widths));

    Self {
      columns,
      widths,
      center,
    }
  }

  /// Total number of tiling windows across all columns.
  pub fn window_count(&self) -> usize {
    self.columns.iter().map(Vec::len).sum()
  }

  /// Id of the window in the center column's top slot, if the layout has
  /// a center.
  pub fn center_window_id(&self) -> Option<Uuid> {
    self
      .columns
      .get(self.center?)
      .and_then(|col| col.first())
      .map(CommonGetters::id)
  }

  /// Current width fraction of the center column, if the layout has a
  /// center.
  pub fn center_width(&self) -> Option<f32> {
    self.widths.get(self.center?).copied()
  }

  /// Window to focus when the window with `id` closes: the next window in
  /// its column, else the previous one, else the top of the nearest other
  /// column (the left one on a tie).
  pub fn neighbor_of(&self, id: Uuid) -> Option<TilingWindow> {
    let (col, row) = self.find(id)?;
    let column = &self.columns[col];

    if let Some(window) = column
      .get(row + 1)
      .or_else(|| row.checked_sub(1).and_then(|above| column.get(above)))
    {
      return Some(window.clone());
    }

    (0..self.columns.len())
      .filter(|&other| other != col && !self.columns[other].is_empty())
      .min_by_key(|&other| (other.abs_diff(col), other))
      .and_then(|other| self.columns[other].first().cloned())
  }

  /// `(column, row)` position of the window with `id`, if present.
  pub fn find(&self, id: Uuid) -> Option<(usize, usize)> {
    self.columns.iter().enumerate().find_map(|(col, windows)| {
      windows
        .iter()
        .position(|window| window.id() == id)
        .map(|row| (col, row))
    })
  }

  /// Rebuilds `workspace` into these columns (left-to-right), sized to
  /// their widths. Empty columns are dropped and widths renormalised.
  /// Records the center column's top window on the workspace.
  ///
  /// Uses only the existing tree primitives — `move_container_within_tree`
  /// to gather windows and `wrap_in_split_container` to form columns — so
  /// windows stay tiled and the workspace is redrawn once at the end (no
  /// flicker).
  #[allow(clippy::cast_precision_loss)]
  pub fn render(
    self,
    workspace: &Workspace,
    state: &mut WmState,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    workspace.set_center_window_id(self.center_window_id());

    // Drop empty columns, pairing each surviving column with its width.
    let mut cols: Vec<Vec<TilingWindow>> = Vec::new();
    let mut fracs: Vec<f32> = Vec::new();
    for (column, width) in self.columns.into_iter().zip(self.widths) {
      if !column.is_empty() {
        cols.push(column);
        fracs.push(width);
      }
    }

    if cols.is_empty() {
      return Ok(());
    }

    // Normalise so column fractions sum to 1.
    let total = fracs.iter().sum::<f32>().max(f32::EPSILON);
    for frac in &mut fracs {
      *frac /= total;
    }

    // Columns sit side-by-side, so the workspace must tile horizontally.
    workspace.set_tiling_direction(TilingDirection::Horizontal);
    let workspace_container: Container = workspace.clone().into();

    // Phase 1: pull every window up to the workspace in the final flat
    // order (columns left-to-right, windows top-to-bottom). Moving windows
    // out of their old split containers flattens those splits away.
    let flat = cols.iter().flatten().cloned().collect::<Vec<_>>();
    for (index, window) in flat.iter().enumerate() {
      let container: Container = window.clone().into();
      move_container_within_tree(
        &container,
        &workspace_container,
        index,
        state,
      )?;
    }

    // Phase 2: wrap each multi-window column into a vertical split. A
    // single window column already sits directly under the workspace.
    let mut column_entities: Vec<Container> = Vec::new();
    for column in &cols {
      if column.len() == 1 {
        column_entities.push(column[0].clone().into());
        continue;
      }

      let split = SplitContainer::new(
        TilingDirection::Vertical,
        config.value.gaps.clone(),
      );

      let children = column
        .iter()
        .map(|window| window.clone().into())
        .collect::<Vec<TilingContainer>>();

      wrap_in_split_container(&split, &workspace_container, &children)?;

      // Even split between the column's rows.
      let row_size = 1.0 / column.len() as f32;
      for window in column {
        window.set_tiling_size(row_size);
      }

      column_entities.push(split.into());
    }

    // Phase 3: set each column's width.
    for (entity, frac) in column_entities.iter().zip(&fracs) {
      if let Ok(tiling) = entity.as_tiling_container() {
        tiling.set_tiling_size(*frac);
      }
    }

    // One atomic redraw of the whole workspace.
    state
      .pending_sync
      .queue_container_to_redraw(workspace_container);

    Ok(())
  }
}

/// Index of the single strictly-widest column, if any. Equal-width columns
/// (a `C`-less layout) have none.
fn strictly_widest(widths: &[f32]) -> Option<usize> {
  let max = widths.iter().copied().reduce(f32::max)?;
  let mut widest = widths
    .iter()
    .enumerate()
    .filter(|(_, &width)| (max - width).abs() < 1e-4)
    .map(|(index, _)| index);

  match (widest.next(), widest.next()) {
    (Some(index), None) => Some(index),
    _ => None,
  }
}
