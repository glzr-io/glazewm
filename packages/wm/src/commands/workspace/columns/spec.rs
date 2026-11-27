//! Parsing and window-distribution for `columns` specs.
//!
//! Pure layout math with no dependency on the window tree, so the
//! assignment can be unit-tested with plain indices in place of live
//! windows.

use wm_common::ColumnBias;

/// One column in a `columns` spec.
#[derive(Debug, PartialEq)]
pub(super) enum ColumnKind {
  /// The wide center column holding the focused window.
  Center,
  /// A stack of exactly `n` windows.
  Fixed(usize),
  /// A stack claiming an even share of the windows left over after the
  /// fixed columns take theirs.
  Star,
}

/// Parses a comma-separated column `spec` into an ordered list of column
/// kinds, left-to-right. A number token is that many stacked windows, `*`
/// is a leftover-sharing stack, and `C` is the wide center.
///
/// Errors on an unrecognised token, a zero fixed count, or a spec with
/// more than one `C`. A `C` is optional: a spec without one lays the
/// windows out as plain equal-width columns with no wide center.
pub(super) fn parse_columns_spec(
  spec: &str,
) -> anyhow::Result<Vec<ColumnKind>> {
  let mut kinds = Vec::new();
  for token in spec.split(',').map(str::trim).filter(|t| !t.is_empty()) {
    if token.eq_ignore_ascii_case("c") {
      kinds.push(ColumnKind::Center);
    } else if token == "*" {
      kinds.push(ColumnKind::Star);
    } else {
      let count = token.parse::<usize>().map_err(|_| {
        anyhow::anyhow!("Column `{token}` must be a number, `*`, or `C`.")
      })?;
      if count == 0 {
        anyhow::bail!("Column count must be at least 1.");
      }
      kinds.push(ColumnKind::Fixed(count));
    }
  }

  if kinds
    .iter()
    .filter(|k| matches!(k, ColumnKind::Center))
    .count()
    > 1
  {
    anyhow::bail!("Column spec must contain at most one `C`.");
  }

  Ok(kinds)
}

/// Distributes `center` and the `rest` of the windows into columns per the
/// parsed `kinds`, in on-screen order.
///
/// `center` (present iff `kinds` has a `C`) fills the `C` column; each
/// fixed column takes its exact count; `*` columns share the leftover
/// windows evenly, with `bias` deciding which end claims the odd window(s)
/// when they don't divide evenly. Any windows still unplaced (fixed counts
/// under-specify the total and there is no `*`) are appended to the last
/// non-center column, so nothing is dropped.
///
/// Generic over the item so the assignment can be unit-tested with plain
/// indices in place of live windows.
pub(super) fn distribute_columns<T>(
  kinds: &[ColumnKind],
  mut center: Option<T>,
  rest: Vec<T>,
  bias: &ColumnBias,
) -> Vec<Vec<T>> {
  // The `*` columns share whatever the fixed columns don't claim.
  let fixed_total = kinds
    .iter()
    .map(|k| match k {
      ColumnKind::Fixed(n) => *n,
      _ => 0,
    })
    .sum::<usize>();
  let star_count = kinds
    .iter()
    .filter(|k| matches!(k, ColumnKind::Star))
    .count();
  let leftover = rest.len().saturating_sub(fixed_total);
  let star_base = leftover.checked_div(star_count).unwrap_or(0);
  let star_extra = leftover.checked_rem(star_count).unwrap_or(0);

  let mut rest_iter = rest.into_iter();
  let mut stars_seen = 0;
  let mut columns = Vec::with_capacity(kinds.len());
  for kind in kinds {
    match kind {
      ColumnKind::Center => columns.push(vec![center
        .take()
        .expect("a `C` column requires a center window")]),
      ColumnKind::Fixed(n) => {
        columns.push(rest_iter.by_ref().take(*n).collect());
      }
      ColumnKind::Star => {
        // The odd leftover windows go to the first `star_extra` stars for
        // a left bias, or the last `star_extra` stars for a right bias.
        let gets_extra = match bias {
          ColumnBias::Left => stars_seen < star_extra,
          ColumnBias::Right => stars_seen >= star_count - star_extra,
        };
        let take = star_base + usize::from(gets_extra);
        stars_seen += 1;
        columns.push(rest_iter.by_ref().take(take).collect());
      }
    }
  }

  let remaining = rest_iter.collect::<Vec<_>>();
  if !remaining.is_empty() {
    let target = kinds
      .iter()
      .rposition(|k| !matches!(k, ColumnKind::Center))
      .unwrap_or(0);
    columns[target].extend(remaining);
  }

  columns
}

/// The width fraction of each column: the center takes `center_fraction`
/// and the remaining width is divided evenly across the non-center
/// columns. With no center column, `center_fraction` is ignored and every
/// column takes an equal `1/n` share.
#[allow(clippy::cast_precision_loss)]
pub(super) fn column_widths(
  kinds: &[ColumnKind],
  center_fraction: f32,
) -> Vec<f32> {
  let has_center = kinds.iter().any(|k| matches!(k, ColumnKind::Center));
  if !has_center {
    let share = if kinds.is_empty() {
      0.0
    } else {
      1.0 / kinds.len() as f32
    };
    return vec![share; kinds.len()];
  }

  let non_center = kinds.len().saturating_sub(1);
  let side = if non_center > 0 {
    (1.0 - center_fraction) / non_center as f32
  } else {
    0.0
  };

  kinds
    .iter()
    .map(|kind| match kind {
      ColumnKind::Center => center_fraction,
      _ => side,
    })
    .collect()
}

/// Clamps a center width to the range the layout supports.
pub(super) fn clamp_center(center: f32) -> f32 {
  center.clamp(0.1, 0.99)
}

/// Width fraction the center column ends up with once laid out: its
/// `column_widths` share after `ColumnGrid::render` drops the empty
/// columns and renormalises the rest.
///
/// Returns `None` when `kinds` has no `C` or there are fewer than two
/// windows, since no center is laid out then.
pub(super) fn rendered_center_width(
  kinds: &[ColumnKind],
  window_count: usize,
  center: f32,
  bias: &ColumnBias,
) -> Option<f32> {
  let center_index =
    kinds.iter().position(|k| matches!(k, ColumnKind::Center))?;

  if window_count < 2 {
    return None;
  }

  let columns =
    distribute_columns(kinds, Some(()), vec![(); window_count - 1], bias);
  let widths = column_widths(kinds, clamp_center(center));

  let total = columns
    .iter()
    .zip(&widths)
    .filter(|(column, _)| !column.is_empty())
    .map(|(_, width)| width)
    .sum::<f32>();

  Some(widths.get(center_index)? / total.max(f32::EPSILON))
}

/// Laid-out widths closer than this count as the same width.
const WIDTH_TOLERANCE: f32 = 1e-3;

/// Index of the center-width candidate to apply next, so repeated calls
/// cycle through every distinct laid-out width in list order.
///
/// `rendered` holds each candidate's laid-out width. The cycle continues
/// after the candidate matching `current`, skipping candidates that lay
/// out to that same width. Starts at the first candidate when `current`
/// is `None` or matches none of them (e.g. another layout or a manual
/// resize).
pub(super) fn next_center_index(
  rendered: &[f32],
  current: Option<f32>,
) -> usize {
  let Some(current) = current else {
    return 0;
  };

  let differs = |width: f32| (width - current).abs() >= WIDTH_TOLERANCE;

  let Some(matched) = rendered.iter().position(|&width| !differs(width))
  else {
    return 0;
  };

  (1..=rendered.len())
    .map(|step| (matched + step) % rendered.len())
    .find(|&index| differs(rendered[index]))
    .unwrap_or(matched)
}

#[cfg(test)]
mod tests {
  use wm_common::ColumnBias;

  use super::{
    column_widths, distribute_columns, next_center_index,
    parse_columns_spec, rendered_center_width, ColumnKind,
  };

  /// Presses a cycling binding `presses` times from no laid-out center,
  /// returning the width applied by each press.
  fn press(rendered: &[f32], presses: usize) -> Vec<f32> {
    let mut current = None;
    (0..presses)
      .map(|_| {
        let width = rendered[next_center_index(rendered, current)];
        current = Some(width);
        width
      })
      .collect()
  }

  /// Laid-out widths for `candidates` under `spec`.
  fn rendered(
    spec: &str,
    window_count: usize,
    candidates: &[f32],
    bias: &ColumnBias,
  ) -> Vec<f32> {
    let kinds = parse_columns_spec(spec).unwrap();
    candidates
      .iter()
      .map(|&c| rendered_center_width(&kinds, window_count, c, bias))
      .map(Option::unwrap)
      .collect()
  }

  #[test]
  fn rendered_width_renormalises_empty_columns() {
    let left = ColumnBias::Left;

    // One side column is empty, so the center gets 0.6 of 0.8.
    assert!((rendered("*,C,*", 2, &[0.6], &left)[0] - 0.75).abs() < 1e-5);
    assert!((rendered("*,C,*", 3, &[0.6], &left)[0] - 0.6).abs() < 1e-5);

    let kinds = parse_columns_spec("*,C,*").unwrap();
    assert_eq!(rendered_center_width(&kinds, 1, 0.6, &left), None);
    let no_center = parse_columns_spec("*,*").unwrap();
    assert_eq!(rendered_center_width(&no_center, 4, 0.6, &left), None);
  }

  /// Regression: `0.6` lays out as `0.75` here, which a raw-value
  /// comparison takes for `0.7`, so the binding never leaves `0.6`.
  #[test]
  fn cycles_renormalised_widths() {
    let widths = rendered("*,C,*", 2, &[0.6, 0.7], &ColumnBias::Left);
    let pressed = press(&widths, 4);
    assert_eq!(pressed, [widths[0], widths[1], widths[0], widths[1]]);
  }

  /// Regression: `1.5` clamps to `0.99`, which a raw-value comparison
  /// takes for `0.7`, so the binding stays on `0.99`.
  #[test]
  fn cycles_clamped_widths() {
    let widths = rendered("C,*", 2, &[0.7, 1.5], &ColumnBias::Left);
    assert_eq!(press(&widths, 3), [widths[0], widths[1], widths[0]]);
  }

  #[test]
  fn starts_at_first_on_unmatched_width() {
    assert_eq!(next_center_index(&[0.7, 0.99], Some(0.5)), 0);
    assert_eq!(next_center_index(&[0.7, 0.99], None), 0);
    assert_eq!(next_center_index(&[], Some(0.5)), 0);
  }

  /// For every spec, window count and candidate list, `n` presses visit
  /// each of the `n` distinct laid-out widths once, in list order, then
  /// repeat.
  #[test]
  fn cycle_visits_every_distinct_width() {
    let specs = ["*,C,*", "C,*", "*,C", "2,C,*", "1,C,2", "*,*,C,*,*"];
    let lists: [&[f32]; 6] = [
      &[0.7, 0.99],
      &[0.6, 0.7],
      &[0.99, 0.5, 0.7],
      &[0.7, 0.7, 0.9],
      &[0.7, 1.5, 0.99],
      &[0.6],
    ];

    for spec in specs {
      for bias in [ColumnBias::Left, ColumnBias::Right] {
        for window_count in 2..=8 {
          for list in lists {
            let widths = rendered(spec, window_count, list, &bias);

            let mut distinct = Vec::<f32>::new();
            for &width in &widths {
              if distinct.iter().all(|d| (d - width).abs() >= 1e-3) {
                distinct.push(width);
              }
            }

            let n = distinct.len();
            let pressed = press(&widths, 2 * n);
            let case = format!("{spec} {bias:?} {window_count} {list:?}");

            for (i, width) in pressed[..n].iter().enumerate() {
              assert!(
                (width - distinct[i]).abs() < 1e-3,
                "{case}: press {i} gave {width}, want {}",
                distinct[i]
              );
            }
            assert_eq!(pressed[..n], pressed[n..], "{case}: not periodic");
          }
        }
      }
    }
  }

  #[test]
  fn parses_star_center_star() {
    assert_eq!(
      parse_columns_spec("*,C,*").unwrap(),
      vec![ColumnKind::Star, ColumnKind::Center, ColumnKind::Star]
    );
  }

  #[test]
  fn parses_explicit_and_lowercase_center() {
    assert_eq!(
      parse_columns_spec("2,1,c,3").unwrap(),
      vec![
        ColumnKind::Fixed(2),
        ColumnKind::Fixed(1),
        ColumnKind::Center,
        ColumnKind::Fixed(3),
      ]
    );
  }

  #[test]
  fn rejects_bad_specs() {
    // More than one center.
    assert!(parse_columns_spec("C,C").is_err());
    // Zero fixed count.
    assert!(parse_columns_spec("0,C").is_err());
    // Unparseable token.
    assert!(parse_columns_spec("x,C").is_err());
  }

  #[test]
  fn accepts_spec_without_center() {
    // A center is optional: no `C` yields plain equal-width columns.
    assert_eq!(
      parse_columns_spec("*,*").unwrap(),
      vec![ColumnKind::Star, ColumnKind::Star]
    );
    assert_eq!(
      parse_columns_spec("2,2").unwrap(),
      vec![ColumnKind::Fixed(2), ColumnKind::Fixed(2)]
    );
  }

  #[test]
  fn widths_are_equal_without_center() {
    // Without a `C`, every column takes an equal share and `center` is
    // ignored — the widths sum to 1 across `n` columns.
    let kinds = parse_columns_spec("*,*,*").unwrap();
    let widths = column_widths(&kinds, 0.6);

    assert_eq!(widths.len(), 3);
    for width in widths {
      assert!((width - 1.0 / 3.0).abs() < f32::EPSILON);
    }
  }

  #[test]
  fn distributes_even_stars() {
    // `*,C,*` with 4 side windows → two even stacks flanking the center.
    let kinds = parse_columns_spec("*,C,*").unwrap();
    let columns = distribute_columns(
      &kinds,
      Some(0),
      vec![1, 2, 3, 4],
      &ColumnBias::Left,
    );
    assert_eq!(columns, vec![vec![1, 2], vec![0], vec![3, 4]]);
  }

  #[test]
  fn bias_breaks_uneven_star_split() {
    let kinds = parse_columns_spec("*,C,*").unwrap();

    // Odd leftover: left bias gives the extra window to the first stack.
    let left = distribute_columns(
      &kinds,
      Some(0),
      vec![1, 2, 3],
      &ColumnBias::Left,
    );
    assert_eq!(left, vec![vec![1, 2], vec![0], vec![3]]);

    // Right bias gives it to the last stack.
    let right = distribute_columns(
      &kinds,
      Some(0),
      vec![1, 2, 3],
      &ColumnBias::Right,
    );
    assert_eq!(right, vec![vec![1], vec![0], vec![2, 3]]);
  }

  #[test]
  fn appends_unplaced_windows_to_last_non_center_column() {
    // `1,C` places one window in the fixed column and no `*` to absorb the
    // rest, so the leftovers land in the last non-center column.
    let kinds = parse_columns_spec("1,C").unwrap();
    let columns = distribute_columns(
      &kinds,
      Some(0),
      vec![1, 2, 3],
      &ColumnBias::Left,
    );
    assert_eq!(columns, vec![vec![1, 2, 3], vec![0]]);
  }

  #[test]
  fn fixed_column_takes_exact_count_and_star_takes_rest() {
    // `2,C,*`: the fixed column takes exactly two windows and the `*`
    // column absorbs the remaining three.
    let kinds = parse_columns_spec("2,C,*").unwrap();
    let columns = distribute_columns(
      &kinds,
      Some(0),
      vec![1, 2, 3, 4, 5],
      &ColumnBias::Left,
    );
    assert_eq!(columns, vec![vec![1, 2], vec![0], vec![3, 4, 5]]);
  }

  #[test]
  fn distributes_without_center() {
    // No `C`: every window flows into the columns and none is pulled out
    // as a privileged center (`center` is `None`).
    let kinds = parse_columns_spec("*,*").unwrap();
    let columns = distribute_columns(
      &kinds,
      None,
      vec![1, 2, 3, 4],
      &ColumnBias::Left,
    );
    assert_eq!(columns, vec![vec![1, 2], vec![3, 4]]);
  }

  #[test]
  fn widths_split_remainder_evenly() {
    let kinds = parse_columns_spec("*,C,*").unwrap();
    let widths = column_widths(&kinds, 0.6);

    assert_eq!(widths.len(), 3);
    assert!((widths[0] - 0.2).abs() < f32::EPSILON);
    assert!((widths[1] - 0.6).abs() < f32::EPSILON);
    assert!((widths[2] - 0.2).abs() < f32::EPSILON);
  }
}
