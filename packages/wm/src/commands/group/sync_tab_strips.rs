use anyhow::Context;
use tracing::warn;
use wm_common::{ParsedConfig, WindowEffectsConfig};
use wm_platform::{CornerStyle, TabStrip, TabStripState};

use crate::{
  models::GroupContainer,
  traits::{CommonGetters, WindowGetters},
  wm_state::WmState,
};

/// Syncs the tab strip of every group in the tree with the group's
/// current state.
///
/// Strips are created on first sync. Failures are logged rather than
/// returned, so that one failing strip doesn't block the others.
pub fn sync_tab_strips(state: &WmState, config: &ParsedConfig) {
  let groups = state
    .root_container
    .descendants()
    .filter_map(|container| container.as_group().cloned())
    .collect::<Vec<_>>();

  for group in groups {
    if let Err(err) = sync_tab_strip(&group, state, config) {
      warn!("Failed to sync tab strip: {err:?}");
    }
  }
}

/// Syncs the tab strip of a single group.
fn sync_tab_strip(
  group: &GroupContainer,
  state: &WmState,
  config: &ParsedConfig,
) -> anyhow::Result<()> {
  let workspace = group.workspace().context("No workspace.")?;
  let groups_config = &config.groups;

  let tab_state = TabStripState {
    rect: group.header_rect()?,
    tab_count: group.child_count(),
    active_index: group.active_index(),
    active_color: groups_config
      .resolved_active_page_color(&config.window_effects),
    inactive_color: groups_config
      .resolved_inactive_page_color(&config.window_effects),
    text_color: groups_config.text_color.clone(),
    corner_radius: corner_radius_px(&config.window_effects),
    show_labels: groups_config.show_labels,
    z_order_after: group
      .active_window()
      .map(|window| window.native().id()),
    is_visible: workspace.is_displayed(),
  };

  let mut tab_strip = group.tab_strip();

  if tab_strip.is_none() {
    *tab_strip = Some(TabStrip::new(&state.dispatcher)?);
  }

  if let Some(tab_strip) = tab_strip.as_mut() {
    tab_strip.set_state(&tab_state)?;
  }

  Ok(())
}

/// Gets the radius in pixels of the rounded corners for windows.
///
/// Mirrors the corner styles applied in `platform_sync`. When the corner
/// effect is disabled, windows keep the Windows 11 default, which is
/// rounded.
fn corner_radius_px(window_effects: &WindowEffectsConfig) -> i32 {
  let corner_style = if window_effects.focused_window.corner_style.enabled
  {
    &window_effects.focused_window.corner_style.style
  } else {
    &CornerStyle::Default
  };

  match corner_style {
    CornerStyle::Default | CornerStyle::Rounded => 8,
    CornerStyle::SmallRounded => 4,
    CornerStyle::Square => 0,
  }
}

#[cfg(test)]
mod tests {
  use wm_common::{CornerEffectConfig, WindowEffectsConfig};
  use wm_platform::CornerStyle;

  use super::corner_radius_px;

  fn effects_with(
    style: CornerStyle,
    enabled: bool,
  ) -> WindowEffectsConfig {
    let mut effects = WindowEffectsConfig::default();
    effects.focused_window.corner_style =
      CornerEffectConfig { enabled, style };
    effects
  }

  #[test]
  fn corner_radius_follows_corner_style() {
    assert_eq!(
      corner_radius_px(&effects_with(CornerStyle::Square, true)),
      0
    );
    assert_eq!(
      corner_radius_px(&effects_with(CornerStyle::Rounded, true)),
      8
    );
    assert_eq!(
      corner_radius_px(&effects_with(CornerStyle::SmallRounded, true)),
      4
    );
  }

  #[test]
  fn corner_radius_defaults_to_rounded_when_effect_disabled() {
    assert_eq!(
      corner_radius_px(&effects_with(CornerStyle::Square, false)),
      8
    );
  }
}
