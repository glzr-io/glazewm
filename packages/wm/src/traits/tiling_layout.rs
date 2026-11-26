use anyhow::Context;
use wm_common::TilingDirection;
use wm_platform::Rect;

use crate::{
  models::TilingContainer,
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters,
    TilingSizeGetters, WindowGetters,
  },
};

#[derive(Clone, Copy, Debug, Default)]
struct LengthConstraint {
  fixed: Option<i32>,
  minimum: i32,
}

/// Propagate fixed dimensions through splits without constraining flexible
/// descendants. An orthogonal split must be wide/tall enough for every
/// child.
fn length_constraint(
  container: &TilingContainer,
  width: bool,
) -> anyhow::Result<LengthConstraint> {
  match container {
    TilingContainer::TilingWindow(window) => {
      let properties = window.native_properties();
      let fixed = if width {
        properties.fixed_size.width
      } else {
        properties.fixed_size.height
      };
      let minimum = if width {
        properties.fixed_size.minimum_width
      } else {
        properties.fixed_size.minimum_height
      };
      let border_delta = window.total_border_delta()?;
      let tile_length = |length| {
        let native_rect = if width {
          Rect::from_xy(0, 0, length, properties.frame.height())
        } else {
          Rect::from_xy(0, 0, properties.frame.width(), length)
        };
        let tile = native_rect.apply_delta(&border_delta.inverse(), None);
        if width {
          tile.width()
        } else {
          tile.height()
        }
      };
      Ok(LengthConstraint {
        fixed: fixed.map(tile_length),
        minimum: fixed.or(minimum).map_or(0, tile_length),
      })
    }
    TilingContainer::Split(split) => {
      let children = split
        .tiling_children()
        .map(|child| length_constraint(&child, width))
        .collect::<anyhow::Result<Vec<_>>>()?;
      if children.is_empty() {
        return Ok(LengthConstraint::default());
      }
      let along_split =
        width == (split.tiling_direction() == TilingDirection::Horizontal);
      let (horizontal_gap, vertical_gap) = split.inner_gaps()?;
      let gap = if width { horizontal_gap } else { vertical_gap };
      Ok(combine_constraints(&children, along_split, gap))
    }
  }
}

fn combine_constraints(
  children: &[LengthConstraint],
  along_split: bool,
  gap: i32,
) -> LengthConstraint {
  if children
    .iter()
    .all(|child| child.fixed.is_none() && child.minimum == 0)
  {
    return LengthConstraint::default();
  }
  #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
  let minimum = if along_split {
    children
      .iter()
      .map(|child| child.minimum.max(1))
      .sum::<i32>()
      + gap * children.len().saturating_sub(1) as i32
  } else {
    children
      .iter()
      .map(|child| child.minimum)
      .max()
      .unwrap_or(0)
  };
  let fixed = (!children.is_empty()
    && children.iter().all(|child| child.fixed.is_some()))
  .then_some(minimum);
  LengthConstraint { fixed, minimum }
}

/// Reserve fixed/minimum lengths and redistribute the remaining space in
/// proportion to the flexible containers' existing tiling weights.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn allocate_lengths(
  available: i32,
  weights: &[f32],
  constraints: &[LengthConstraint],
  horizontal: bool,
) -> Vec<i32> {
  let minimum_total: i64 = constraints
    .iter()
    .map(|constraint| i64::from(constraint.minimum.max(1)))
    .sum();
  if minimum_total > i64::from(available)
    || constraints
      .iter()
      .all(|c| c.minimum == 0 && c.fixed.is_none())
  {
    // If native bounds cannot fit, use the default proportional layout.
    // Apps may overlap their tiles, but must not push sibling tiles away.
    let total_weight: f32 = weights.iter().sum();
    let mut weight_sum = 0.0;
    let mut assigned = 0;
    return weights
      .iter()
      .map(|weight| {
        weight_sum += weight;
        let end = if total_weight > 0.0 {
          let length = available.max(0) as f32 * weight_sum / total_weight;
          if horizontal {
            length.round() as i32
          } else {
            length as i32
          }
        } else {
          0
        };
        let length = end - assigned;
        assigned = end;
        length
      })
      .collect();
  }

  let mut lengths = vec![0; weights.len()];
  let mut flexible = Vec::new();
  let mut remaining = available;
  for (index, constraint) in constraints.iter().enumerate() {
    if let Some(fixed) = constraint.fixed {
      lengths[index] = fixed;
      remaining -= fixed;
    } else {
      flexible.push(index);
    }
  }

  loop {
    let total_weight: f32 = flexible.iter().map(|i| weights[*i]).sum();
    let mut reserved = false;
    let available_share = remaining.max(0);
    flexible.retain(|index| {
      let share = if total_weight > 0.0 {
        available_share as f32 * weights[*index] / total_weight
      } else {
        0.0
      };
      let minimum = constraints[*index].minimum.max(1);
      if share < minimum as f32 {
        lengths[*index] = minimum;
        remaining -= minimum;
        reserved = true;
        false
      } else {
        true
      }
    });
    if !reserved {
      break;
    }
  }

  // Cumulative rounding distributes every remaining pixel without drift.
  let total_weight: f32 = flexible.iter().map(|i| weights[*i]).sum();
  let mut weight_sum = 0.0;
  let mut assigned = 0;
  for index in flexible {
    weight_sum += weights[index];
    let end =
      (remaining.max(0) as f32 * weight_sum / total_weight).round() as i32;
    lengths[index] = end - assigned;
    assigned = end;
  }
  lengths
}

/// Compute a tile using native fixed dimensions rather than stretching
/// them to their proportional share. This also fits lone and orthogonal
/// tiles.
pub fn fitted_tiling_rect(
  container: &TilingContainer,
) -> anyhow::Result<Rect> {
  let parent = container
    .parent()
    .and_then(|parent| parent.as_direction_container().ok())
    .context("No tiling parent.")?;
  let parent_rect = parent.to_rect()?;
  let horizontal =
    parent.tiling_direction() == TilingDirection::Horizontal;
  let children = parent.tiling_children().collect::<Vec<_>>();
  let index = children
    .iter()
    .position(|child| child.id() == container.id())
    .context("Tile is missing from its parent.")?;
  let (horizontal_gap, vertical_gap) = container.inner_gaps()?;
  let axis_length = if horizontal {
    parent_rect.width()
  } else {
    parent_rect.height()
  }
  .max(0);
  #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
  let gap_count = children.len().saturating_sub(1) as i32;
  let gap = if horizontal {
    horizontal_gap
  } else {
    vertical_gap
  }
  .max(0)
  .min(axis_length / gap_count.max(1));
  let available = axis_length - gap * gap_count;
  let constraints = children
    .iter()
    .map(|child| length_constraint(child, horizontal))
    .collect::<anyhow::Result<Vec<_>>>()?;
  let weights = children
    .iter()
    .map(TilingSizeGetters::tiling_size)
    .collect::<Vec<_>>();
  let lengths =
    allocate_lengths(available, &weights, &constraints, horizontal);
  let cross_constraint = length_constraint(container, !horizontal)?;
  let cross_available = if horizontal {
    parent_rect.height()
  } else {
    parent_rect.width()
  };
  let cross_length = cross_constraint
    .fixed
    .unwrap_or(cross_available)
    .min(cross_available);
  Ok(tile_rect(
    &parent_rect,
    &lengths,
    index,
    gap,
    cross_length,
    horizontal,
  ))
}

fn tile_rect(
  parent: &Rect,
  lengths: &[i32],
  index: usize,
  gap: i32,
  cross_length: i32,
  horizontal: bool,
) -> Rect {
  #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
  let offset = lengths[..index].iter().sum::<i32>() + gap * index as i32;
  if horizontal {
    Rect::from_xy(
      parent.x() + offset,
      parent.y(),
      lengths[index],
      cross_length,
    )
  } else {
    Rect::from_xy(
      parent.x(),
      parent.y() + offset,
      cross_length,
      lengths[index],
    )
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  fn fixed(length: i32) -> LengthConstraint {
    LengthConstraint {
      fixed: Some(length),
      minimum: length,
    }
  }

  #[test]
  fn crowded_proportional_layout_does_not_accumulate_rounding_overflow() {
    for horizontal in [true, false] {
      for count in [3_u16, 7, 15, 51] {
        let weight = 1.0 / f32::from(count);
        let count = usize::from(count);
        let weights = vec![weight; count];
        let constraints = vec![fixed(700); count];
        let lengths =
          allocate_lengths(1001, &weights, &constraints, horizontal);
        assert_eq!(lengths.iter().sum::<i32>(), 1001);
        assert!(lengths.iter().all(|length| *length >= 0));
      }
    }
  }

  #[test]
  fn overflowing_minimums_use_default_horizontal_tiles() {
    let constraints =
      [770, 694, 360, 200, 500].map(|minimum| LengthConstraint {
        fixed: None,
        minimum,
      });
    let lengths = allocate_lengths(
      1600,
      &[0.1, 0.2, 0.2, 0.25, 0.25],
      &constraints,
      true,
    );
    assert_eq!(lengths, [160, 320, 320, 400, 400]);
    assert_eq!(
      tile_rect(
        &Rect::from_xy(5, 45, 1640, 1067),
        &lengths,
        4,
        10,
        1067,
        true
      ),
      Rect::from_xy(1245, 45, 400, 1067)
    );
  }

  #[test]
  fn overflowing_minimums_use_default_vertical_tiles() {
    let constraints =
      [620, 650, 400, 150, 470].map(|minimum| LengthConstraint {
        fixed: None,
        minimum,
      });
    let lengths = allocate_lengths(1030, &[0.2; 5], &constraints, false);
    assert_eq!(lengths, [206; 5]);
    assert_eq!(
      tile_rect(
        &Rect::from_xy(5, 45, 1718, 1070),
        &lengths,
        4,
        10,
        1718,
        false
      ),
      Rect::from_xy(5, 909, 1718, 206)
    );
  }

  #[test]
  fn oversized_fixed_and_nested_constraints_do_not_expand_tiles() {
    assert_eq!(
      allocate_lengths(1000, &[1.0], &[fixed(2000)], true),
      [1000]
    );
    let nested = combine_constraints(&[fixed(600), fixed(600)], true, 10);
    assert_eq!(
      allocate_lengths(
        1000,
        &[0.5, 0.5],
        &[nested, LengthConstraint::default()],
        true
      ),
      [500, 500]
    );
  }
  #[test]
  fn minimums_fit_vertically_but_not_horizontally() {
    let widths = [860, 694, 500, 380].map(|minimum| LengthConstraint {
      fixed: None,
      minimum,
    });
    let heights = [LengthConstraint {
      fixed: None,
      minimum: 180,
    }; 4];
    assert!(combine_constraints(&widths, true, 10).minimum > 1718);
    assert!(combine_constraints(&widths, false, 10).minimum <= 1718);
    assert!(combine_constraints(&heights, true, 10).minimum <= 1067);
    let lengths = allocate_lengths(1037, &[0.25; 4], &heights, false);
    for index in 0..4 {
      let rect = tile_rect(
        &Rect::from_xy(5, 45, 1718, 1067),
        &lengths,
        index,
        10,
        1718,
        false,
      );
      assert!(rect.top >= 45 && rect.bottom <= 1112);
      assert!(rect.height() >= 180);
    }
  }

  #[test]
  fn neither_direction_fits_when_both_axes_are_overconstrained() {
    let widths = [860, 694, 500, 380].map(|minimum| LengthConstraint {
      fixed: None,
      minimum,
    });
    let heights = [LengthConstraint {
      fixed: None,
      minimum: 400,
    }; 4];
    assert!(combine_constraints(&widths, true, 10).minimum > 1718);
    assert!(combine_constraints(&heights, true, 10).minimum > 1067);
    let nested = combine_constraints(&[fixed(1800)], false, 10);
    assert!(combine_constraints(&[nested], true, 10).minimum > 1718);
    assert!(combine_constraints(&[nested], false, 10).minimum > 1718);
  }

  #[test]
  fn minimum_width_moves_a_narrow_tile_inside_the_monitor() {
    let lengths = allocate_lengths(
      1698,
      &[1.0 / 3.0; 3],
      &[
        LengthConstraint::default(),
        LengthConstraint::default(),
        LengthConstraint {
          fixed: None,
          minimum: 1200,
        },
      ],
      true,
    );
    assert_eq!(lengths, vec![249, 249, 1200]);
    assert_eq!(
      tile_rect(
        &Rect::from_xy(5, 45, 1718, 1067),
        &lengths,
        2,
        10,
        1067,
        true
      ),
      Rect::from_xy(523, 45, 1200, 1067)
    );
    assert_eq!(
      allocate_lengths(
        3000,
        &[0.5, 0.5],
        &[
          LengthConstraint {
            fixed: None,
            minimum: 1200
          },
          LengthConstraint::default()
        ],
        true
      ),
      vec![1500, 1500]
    );
  }

  #[test]
  fn nested_minimum_width_remains_flexible_above_its_bound() {
    let nested = combine_constraints(
      &[
        LengthConstraint {
          fixed: None,
          minimum: 1200,
        },
        LengthConstraint::default(),
      ],
      false,
      10,
    );
    assert_eq!(nested.fixed, None);
    assert_eq!(nested.minimum, 1200);
    assert_eq!(
      allocate_lengths(
        1700,
        &[0.5, 0.5],
        &[nested, LengthConstraint::default()],
        true
      ),
      vec![1200, 500]
    );
    assert_eq!(
      allocate_lengths(
        3000,
        &[0.5, 0.5],
        &[nested, LengthConstraint::default()],
        true
      ),
      vec![1500, 1500]
    );
  }

  #[test]
  fn fixed_width_tile_returns_space_to_its_neighbor() {
    let lengths = allocate_lengths(
      3420,
      &[0.5, 0.5],
      &[fixed(740), LengthConstraint::default()],
      true,
    );
    assert_eq!(lengths, vec![740, 2680]);
    let parent = Rect::from_xy(1733, 45, 3430, 1390);
    assert_eq!(
      tile_rect(&parent, &lengths, 0, 10, 1390, true),
      Rect::from_xy(1733, 45, 740, 1390)
    );
    assert_eq!(
      tile_rect(&parent, &lengths, 1, 10, 1390, true),
      Rect::from_xy(2483, 45, 2680, 1390)
    );
  }

  #[test]
  fn lone_fixed_width_tile_does_not_expand_to_fill_workspace() {
    let lengths = allocate_lengths(3430, &[1.0], &[fixed(740)], true);
    let parent = Rect::from_xy(1733, 45, 3430, 1390);
    assert_eq!(
      tile_rect(&parent, &lengths, 0, 10, 1390, true),
      Rect::from_xy(1733, 45, 740, 1390)
    );
  }

  #[test]
  fn fixed_width_also_fits_in_a_vertical_layout() {
    let lengths = allocate_lengths(
      1380,
      &[0.5, 0.5],
      &[LengthConstraint::default(); 2],
      false,
    );
    let parent = Rect::from_xy(1733, 45, 3430, 1390);
    assert_eq!(
      tile_rect(&parent, &lengths, 0, 10, 740, false),
      Rect::from_xy(1733, 45, 740, 690)
    );
    assert_eq!(
      tile_rect(&parent, &lengths, 1, 10, 3430, false),
      Rect::from_xy(1733, 745, 3430, 690)
    );
  }

  #[test]
  fn nested_split_reserves_enough_width_for_its_fixed_child() {
    let split = combine_constraints(
      &[fixed(1000), LengthConstraint::default()],
      false,
      10,
    );
    assert_eq!(split.minimum, 1000);
    assert_eq!(split.fixed, None);
    assert_eq!(
      allocate_lengths(
        1660,
        &[0.5, 0.5],
        &[split, LengthConstraint::default()],
        true
      ),
      vec![1000, 660]
    );
  }

  #[test]
  fn unconstrained_splits_preserve_the_original_layout() {
    let split =
      combine_constraints(&[LengthConstraint::default(); 2], true, 10);
    assert_eq!(split.minimum, 0);
    assert_eq!(split.fixed, None);
    assert_eq!(
      allocate_lengths(1001, &[0.5, 0.5], &[split; 2], true),
      vec![501, 500]
    );
    assert_eq!(
      allocate_lengths(1001, &[0.5, 0.5], &[split; 2], false),
      vec![500, 501]
    );
  }

  #[test]
  fn fully_fixed_nested_splits_propagate_their_dimensions() {
    let children = [fixed(400), fixed(300)];
    assert_eq!(combine_constraints(&children, true, 10).fixed, Some(710));
    assert_eq!(combine_constraints(&children, false, 10).fixed, Some(400));
  }

  #[test]
  fn fixed_height_returns_space_to_vertical_neighbor() {
    let lengths = allocate_lengths(
      1380,
      &[0.5, 0.5],
      &[fixed(300), LengthConstraint::default()],
      false,
    );
    assert_eq!(lengths, vec![300, 1080]);
    let parent = Rect::from_xy(1733, 45, 3430, 1390);
    assert_eq!(
      tile_rect(&parent, &lengths, 0, 10, 3430, false),
      Rect::from_xy(1733, 45, 3430, 300)
    );
    assert_eq!(
      tile_rect(&parent, &lengths, 1, 10, 3430, false),
      Rect::from_xy(1733, 355, 3430, 1080)
    );
  }

  #[test]
  fn flexible_weights_and_rounding_preserve_total_space() {
    let constraints = [
      LengthConstraint {
        fixed: Some(740),
        minimum: 740,
      },
      LengthConstraint::default(),
      LengthConstraint::default(),
    ];
    assert_eq!(
      allocate_lengths(1741, &[0.5, 0.3, 0.2], &constraints, true),
      vec![740, 601, 400]
    );
  }

  #[test]
  fn all_fixed_tiles_leave_unused_space_instead_of_stretching() {
    let constraints = [LengthConstraint {
      fixed: Some(300),
      minimum: 300,
    }; 2];
    assert_eq!(
      allocate_lengths(1000, &[0.5, 0.5], &constraints, true),
      vec![300, 300]
    );
  }

  #[test]
  fn insufficient_space_preserves_proportional_shares() {
    let constraints = [
      LengthConstraint {
        fixed: Some(740),
        minimum: 740,
      },
      LengthConstraint::default(),
    ];
    assert_eq!(
      allocate_lengths(600, &[0.5, 0.5], &constraints, true),
      vec![300, 300]
    );
  }
}
