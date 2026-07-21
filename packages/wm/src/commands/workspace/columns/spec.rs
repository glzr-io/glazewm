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

#[cfg(test)]
mod tests {
  use wm_common::ColumnBias;

  use super::{
    column_widths, distribute_columns, parse_columns_spec, ColumnKind,
  };

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
