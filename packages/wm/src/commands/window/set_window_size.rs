use anyhow::Context;
use wm_common::{TilingDirection, TilingLayout, WindowState};
use wm_platform::{LengthValue, Rect};

use crate::{
  commands::container::resize_tiling_container,
  models::{
    NonTilingWindow, TilingContainer, TilingWindow, WindowContainer,
  },
  traits::{
    accordion_axis_bounds, CommonGetters, PositionGetters,
    TilingDirectionGetters, TilingLayoutGetters, TilingSizeGetters,
    WindowGetters,
  },
  wm_state::WmState,
};

/// Arbitrary defaults for minimum floating window dimensions.
const MIN_FLOATING_WIDTH: i32 = 250;
const MIN_FLOATING_HEIGHT: i32 = 140;

struct AccordionResizeStep {
  child: TilingContainer,
  child_index: usize,
  child_count: usize,
}

pub fn set_window_size(
  window: WindowContainer,
  target_width: Option<LengthValue>,
  target_height: Option<LengthValue>,
  state: &mut WmState,
) -> anyhow::Result<()> {
  match window {
    WindowContainer::TilingWindow(window) => {
      set_tiling_window_size(&window, target_width, target_height, state)?;
    }
    WindowContainer::NonTilingWindow(window) => {
      if matches!(window.state(), WindowState::Floating(_)) {
        set_floating_window_size(
          &window,
          target_width,
          target_height,
          state,
        )?;
      }
    }
  }

  Ok(())
}

fn set_tiling_window_size(
  window: &TilingWindow,
  target_width: Option<LengthValue>,
  target_height: Option<LengthValue>,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if let Some(target_width) = target_width {
    set_tiling_window_length(window, &target_width, true, state)?;
  }

  if let Some(target_height) = target_height {
    set_tiling_window_length(window, &target_height, false, state)?;
  }

  Ok(())
}

/// Updates either the width or height of a tiling window.
fn set_tiling_window_length(
  window: &TilingWindow,
  target_length: &LengthValue,
  is_width_resize: bool,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // When resizing a tiling window, the container to resize can actually be
  // an ancestor split container.
  let container_to_resize = window.container_to_resize(is_width_resize)?;

  if let Some(container_to_resize) = container_to_resize {
    let parent = container_to_resize.parent().context("No parent.")?;
    let container_rect = container_to_resize.to_rect()?;
    let (horizontal_gap, vertical_gap) =
      container_to_resize.inner_gaps()?;
    let sibling_count = container_to_resize.tiling_siblings().count();

    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    let parent_length = if is_width_resize {
      parent.to_rect()?.width() - horizontal_gap * sibling_count as i32
    } else {
      parent.to_rect()?.height() - vertical_gap * sibling_count as i32
    };

    let container_length = if is_width_resize {
      container_rect.width()
    } else {
      container_rect.height()
    };

    let target_window_length = target_length.to_px(parent_length, None);
    let target_container_length = resolve_target_container_length(
      window,
      &container_to_resize,
      target_window_length,
      container_length,
      parent_length,
      is_width_resize,
    )?;

    let tiling_size = match target_container_length {
      Some(target_container_length) => {
        LengthValue::from_px(target_container_length)
          .to_percentage(parent_length)
      }
      None => target_length.to_percentage(parent_length),
    };

    // Skip the resize if the window is already at the target size.
    if container_to_resize.tiling_size() - tiling_size != 0. {
      resize_tiling_container(&container_to_resize, tiling_size);

      state
        .pending_sync
        .queue_containers_to_redraw(parent.tiling_children());
    }
  }

  Ok(())
}

/// Maps a requested leaf-window length to the length of the ancestor that
/// controls the requested axis.
///
/// Accordion padding can be percentage-based and is clamped by the current
/// parent length, so the difference between the leaf and its ancestor is
/// not constant. Search the available integer-pixel lengths against the
/// same geometry function used for layout. This also handles multiple
/// accordion ancestors and the one-pixel transitions introduced by padding
/// clamping.
fn resolve_target_container_length(
  window: &TilingWindow,
  container_to_resize: &TilingContainer,
  target_window_length: i32,
  current_container_length: i32,
  parent_length: i32,
  is_width_resize: bool,
) -> anyhow::Result<Option<i32>> {
  let requested_direction = if is_width_resize {
    TilingDirection::Horizontal
  } else {
    TilingDirection::Vertical
  };

  let resize_steps = accordion_resize_steps(
    window,
    container_to_resize,
    &requested_direction,
  )?;

  if resize_steps.is_empty() {
    return Ok(None);
  }

  let max_container_length =
    parent_length.max(current_container_length).max(0);
  let current_container_length =
    current_container_length.clamp(0, max_container_length);

  let current_window_length =
    accordion_window_length(current_container_length, &resize_steps)?;
  let mut best_length = current_container_length;
  let mut best_score = (
    (i64::from(current_window_length) - i64::from(target_window_length))
      .abs(),
    0,
  );

  for candidate_length in 0..=max_container_length {
    let window_length =
      accordion_window_length(candidate_length, &resize_steps)?;
    let score = (
      (i64::from(window_length) - i64::from(target_window_length)).abs(),
      (i64::from(candidate_length) - i64::from(current_container_length))
        .abs(),
    );

    if score < best_score {
      best_length = candidate_length;
      best_score = score;
    }
  }

  Ok(Some(best_length))
}

/// Accordion relations between the leaf and the ancestor being resized,
/// ordered from the innermost relation to the outermost.
fn accordion_resize_steps(
  window: &TilingWindow,
  container_to_resize: &TilingContainer,
  requested_direction: &TilingDirection,
) -> anyhow::Result<Vec<AccordionResizeStep>> {
  let mut child = window.as_tiling_container()?;
  let mut steps = Vec::new();

  while child.id() != container_to_resize.id() {
    let parent = child
      .parent()
      .and_then(|parent| parent.as_direction_container().ok())
      .context("No direction parent before resize target.")?;

    if parent.tiling_layout() == TilingLayout::Accordion
      && &parent.tiling_direction() == requested_direction
    {
      let tiling_children = parent.tiling_children().collect::<Vec<_>>();

      if tiling_children.len() > 1 {
        let child_index = tiling_children
          .iter()
          .position(|tiling_child| tiling_child.id() == child.id())
          .context("Container is not a tiling child of its parent.")?;

        steps.push(AccordionResizeStep {
          child: child.clone(),
          child_index,
          child_count: tiling_children.len(),
        });
      }
    }

    child = parent
      .as_tiling_container()
      .context("Resize target is not an ancestor of the window.")?;
  }

  Ok(steps)
}

fn accordion_window_length(
  mut container_length: i32,
  resize_steps: &[AccordionResizeStep],
) -> anyhow::Result<i32> {
  for step in resize_steps.iter().rev() {
    let padding = step.child.accordion_padding(container_length)?;
    // Focus changes which edge receives the inset, but not the resulting
    // child length, so a fixed focused index is sufficient here.
    container_length = accordion_axis_bounds(
      0,
      container_length,
      padding,
      step.child_index,
      step.child_count,
      0,
    )
    .1;
  }

  Ok(container_length)
}

fn set_floating_window_size(
  window: &NonTilingWindow,
  target_width: Option<LengthValue>,
  target_height: Option<LengthValue>,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let monitor = window.monitor().context("No monitor")?;
  let monitor_rect = monitor.to_rect()?;
  let window_rect = window.to_rect()?;

  // Prevent resize from making the window smaller than minimum dimensions.
  // Always allow the size to be increased, even if the window would still
  // be within minimum dimension values.
  let length_with_clamp =
    |target_length: Option<i32>, current_length, min_length| {
      target_length.map_or(current_length, |target_length| {
        if target_length >= current_length {
          target_length
        } else {
          target_length.max(min_length)
        }
      })
    };

  let target_width_px = target_width
    .map(|target_width| target_width.to_px(monitor_rect.width(), None));

  let new_width = length_with_clamp(
    target_width_px,
    window_rect.width(),
    MIN_FLOATING_WIDTH,
  );

  let target_height_px = target_height
    .map(|target_height| target_height.to_px(monitor_rect.height(), None));

  let new_height = length_with_clamp(
    target_height_px,
    window_rect.height(),
    MIN_FLOATING_HEIGHT,
  );

  window.set_floating_placement(Rect::from_xy(
    window.floating_placement().x(),
    window.floating_placement().y(),
    new_width,
    new_height,
  ));

  state.pending_sync.queue_container_to_redraw(window.clone());

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::{GapsConfig, TilingDirection, TilingLayout};
  use wm_platform::{LengthUnit, LengthValue};

  use super::resolve_target_container_length;
  use crate::{
    commands::container::resize_tiling_container,
    models::{
      Monitor, SplitContainer, TilingContainer, TilingWindow, Workspace,
    },
    traits::{PositionGetters, TilingLayoutGetters},
  };

  fn gaps_with_accordion_padding(
    accordion_padding: LengthValue,
  ) -> GapsConfig {
    GapsConfig {
      scale_with_dpi: false,
      accordion_padding,
      ..GapsConfig::default()
    }
  }

  fn horizontal_accordion_with_center_child(
    center_child: TilingContainer,
    gaps: &GapsConfig,
  ) -> SplitContainer {
    let split = SplitContainer::mock()
      .gaps_config(gaps.clone())
      .tiling_containers(vec![
        TilingWindow::mock().gaps_config(gaps.clone()).call().into(),
        center_child,
        TilingWindow::mock().gaps_config(gaps.clone()).call().into(),
      ])
      .call();
    split.set_tiling_layout(TilingLayout::Accordion);
    split
  }

  fn attach_resizable_group(
    group: &SplitContainer,
    gaps: &GapsConfig,
  ) -> (Monitor, TilingContainer) {
    let resize_target: TilingContainer = group.clone().into();
    let workspace = Workspace::mock()
      .tiling_direction(TilingDirection::Horizontal)
      .gaps_config(gaps.clone())
      .tiling_containers(vec![
        resize_target.clone(),
        TilingWindow::mock().gaps_config(gaps.clone()).call().into(),
      ])
      .call();
    let monitor = Monitor::mock().workspaces(vec![workspace]).call();
    (monitor, resize_target)
  }

  fn apply_container_length(
    resize_target: &TilingContainer,
    target_length: i32,
    parent_length: i32,
  ) {
    let tiling_size =
      LengthValue::from_px(target_length).to_percentage(parent_length);
    resize_tiling_container(resize_target, tiling_size);
  }

  #[test]
  fn percentage_padding_is_recomputed_for_target_length() {
    let gaps = gaps_with_accordion_padding(LengthValue {
      amount: 0.1,
      unit: LengthUnit::Percentage,
    });
    let window = TilingWindow::mock().gaps_config(gaps.clone()).call();
    let accordion =
      horizontal_accordion_with_center_child(window.clone().into(), &gaps);
    let (_monitor, resize_target) =
      attach_resizable_group(&accordion, &gaps);
    let parent_length = 1680;
    let current_length = resize_target
      .to_rect()
      .expect("Resize target should have a rect.")
      .width();

    let target_length = resolve_target_container_length(
      &window,
      &resize_target,
      336,
      current_length,
      parent_length,
      true,
    )
    .expect("Target length lookup should succeed.")
    .expect("Accordion target length should resolve.");

    assert_eq!(target_length, 420);
    apply_container_length(&resize_target, target_length, parent_length);
    assert_eq!(
      window
        .to_rect()
        .expect("Window should have a rect.")
        .width(),
      336
    );
  }

  #[test]
  fn percentage_padding_is_recomputed_through_multiple_ancestors() {
    let gaps = gaps_with_accordion_padding(LengthValue {
      amount: 0.1,
      unit: LengthUnit::Percentage,
    });
    let window = TilingWindow::mock().gaps_config(gaps.clone()).call();
    let inner_accordion =
      horizontal_accordion_with_center_child(window.clone().into(), &gaps);
    let outer_accordion = horizontal_accordion_with_center_child(
      inner_accordion.into(),
      &gaps,
    );
    let (_monitor, resize_target) =
      attach_resizable_group(&outer_accordion, &gaps);
    let parent_length = 1680;
    let current_length = resize_target
      .to_rect()
      .expect("Resize target should have a rect.")
      .width();

    let target_length = resolve_target_container_length(
      &window,
      &resize_target,
      320,
      current_length,
      parent_length,
      true,
    )
    .expect("Target length lookup should succeed.")
    .expect("Accordion target length should resolve.");

    assert_eq!(target_length, 500);
    apply_container_length(&resize_target, target_length, parent_length);
    assert_eq!(
      window
        .to_rect()
        .expect("Window should have a rect.")
        .width(),
      320
    );
  }

  #[test]
  fn fixed_padding_searches_across_clamp_transition() {
    let gaps = gaps_with_accordion_padding(LengthValue::from_px(100));
    let window = TilingWindow::mock().gaps_config(gaps.clone()).call();
    let accordion =
      horizontal_accordion_with_center_child(window.clone().into(), &gaps);
    let (_monitor, resize_target) =
      attach_resizable_group(&accordion, &gaps);
    let parent_length = 1680;
    apply_container_length(&resize_target, 40, parent_length);

    assert_eq!(
      window
        .to_rect()
        .expect("Window should have a rect.")
        .width(),
      2
    );
    assert_eq!(
      resolve_target_container_length(
        &window,
        &resize_target,
        2,
        40,
        parent_length,
        true,
      )
      .expect("Current target length lookup should succeed.")
      .expect("Current accordion target length should resolve."),
      40
    );

    let target_length = resolve_target_container_length(
      &window,
      &resize_target,
      3,
      40,
      parent_length,
      true,
    )
    .expect("Target length lookup should succeed.")
    .expect("Target length should resolve across the clamp transition.");

    assert_eq!(target_length, 203);
    apply_container_length(&resize_target, target_length, parent_length);
    assert_eq!(
      window
        .to_rect()
        .expect("Window should have a rect.")
        .width(),
      3
    );
  }
}
