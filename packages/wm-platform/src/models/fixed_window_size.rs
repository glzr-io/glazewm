/// Observed fixed dimensions and lower bounds of a native window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FixedWindowSize {
  pub width: Option<i32>,
  pub height: Option<i32>,
  pub minimum_width: Option<i32>,
  pub minimum_height: Option<i32>,
}

impl FixedWindowSize {
  /// Only accept a fixed dimension when both probes agree with the
  /// original size. Retain a lower bound when only shrinking fails;
  /// that dimension must remain resizable above the observed bound.
  #[must_use]
  pub fn from_probes(
    original: (i32, i32),
    larger: (i32, i32),
    smaller: (i32, i32),
    probe_width: bool,
    probe_height: bool,
  ) -> Self {
    Self {
      minimum_width: (probe_width
        && larger.0 > original.0
        && smaller.0 >= original.0)
        .then_some(original.0),
      minimum_height: (probe_height
        && larger.1 > original.1
        && smaller.1 >= original.1)
        .then_some(original.1),
      width: (probe_width
        && original.0 == larger.0
        && original.0 == smaller.0)
        .then_some(original.0),
      height: (probe_height
        && original.1 == larger.1
        && original.1 == smaller.1)
        .then_some(original.1),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detects_fixed_width_without_freezing_height() {
    assert_eq!(
      FixedWindowSize::from_probes(
        (740, 1000),
        (740, 1100),
        (740, 900),
        true,
        true,
      ),
      FixedWindowSize {
        width: Some(740),
        height: None,
        ..FixedWindowSize::default()
      },
    );
  }

  #[test]
  fn one_sided_size_limits_are_not_fixed() {
    for (larger, smaller) in [
      ((840, 1100), (740, 1000)),
      ((740, 1000), (640, 900)),
      ((840, 1100), (640, 900)),
    ] {
      let detected = FixedWindowSize::from_probes(
        (740, 1000),
        larger,
        smaller,
        true,
        true,
      );
      assert_eq!(detected.width, None);
      assert_eq!(detected.height, None);
    }
  }

  #[test]
  fn minimum_width_stays_resizable_above_the_limit() {
    let constraint = FixedWindowSize::from_probes(
      (1200, 900),
      (1300, 1000),
      (1200, 800),
      true,
      true,
    );
    assert_eq!(constraint.width, None);
    assert_eq!(constraint.minimum_width, Some(1200));
    assert_eq!(constraint.minimum_height, None);
  }

  #[test]
  fn minimum_height_does_not_freeze_a_resizable_width() {
    assert_eq!(
      FixedWindowSize::from_probes(
        (1200, 900),
        (1300, 1000),
        (1100, 900),
        true,
        true,
      ),
      FixedWindowSize {
        minimum_height: Some(900),
        ..FixedWindowSize::default()
      },
    );
  }

  #[test]
  fn upper_limits_do_not_create_lower_bounds() {
    assert_eq!(
      FixedWindowSize::from_probes(
        (1200, 900),
        (1200, 900),
        (1100, 800),
        true,
        true,
      ),
      FixedWindowSize::default(),
    );
  }

  #[test]
  fn only_classifies_dimensions_that_were_probed() {
    assert_eq!(
      FixedWindowSize::from_probes(
        (740, 1000),
        (740, 1000),
        (740, 1000),
        false,
        true,
      ),
      FixedWindowSize {
        width: None,
        height: Some(1000),
        ..FixedWindowSize::default()
      },
    );
  }
}
