use std::cell::Ref;

use ambassador::delegatable_trait;
use wm_common::{ActiveDrag, DisplayState, WindowRuleConfig, WindowState};
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;
use wm_platform::{LengthValue, NativeWindow, Rect, RectDelta};

use crate::{
  models::{NativeWindowProperties, Workspace},
  traits::CommonGetters,
  user_config::UserConfig,
};

#[delegatable_trait]
pub trait WindowGetters: CommonGetters {
  fn state(&self) -> WindowState;

  fn set_state(&self, state: WindowState);

  fn prev_state(&self) -> Option<WindowState>;

  fn set_prev_state(&self, state: WindowState);

  /// Gets the "toggled" window state based on the current state and a
  /// given target state.
  ///
  /// This will return the first valid state in the following order:
  /// 1. If the window is not currently in the target state, return the
  ///    target state.
  /// 2. The previous state exists if one exists.
  /// 3. The state from `window_behavior.initial_state` in the user config.
  /// 4. Default to either floating/tiling depending on the current state.
  fn toggled_state(
    &self,
    target_state: WindowState,
    config: &UserConfig,
  ) -> WindowState {
    let possible_states = [
      Some(target_state),
      self.prev_state(),
      Some(WindowState::default_from_config(&config.value)),
    ];

    // Return the first possible state with a different discriminant.
    possible_states
      .into_iter()
      .find_map(|state| {
        state.filter(|state| !self.state().is_same_state(state))
      })
      // Default to tiling from a non-tiling state, and floating from a
      // tiling state.
      .unwrap_or_else(|| match self.state() {
        WindowState::Tiling => WindowState::Floating(
          config.value.window_behavior.state_defaults.floating.clone(),
        ),
        _ => WindowState::Tiling,
      })
  }

  fn native(&self) -> Ref<'_, NativeWindow>;

  fn border_delta(&self) -> RectDelta;

  fn set_border_delta(&self, border_delta: RectDelta);

  fn total_border_delta(&self) -> anyhow::Result<RectDelta> {
    let border_delta = self.border_delta();

    let shadow_border_delta = {
      #[cfg(target_os = "windows")]
      {
        // TODO: Avoid re-querying for shadow borders.
        self.native().shadow_borders()?
      }
      #[cfg(not(target_os = "windows"))]
      {
        RectDelta::zero()
      }
    };

    // TODO: Allow percentage length values.
    Ok(RectDelta {
      left: LengthValue::from_px(
        border_delta.left.to_px(0, None)
          + shadow_border_delta.left.to_px(0, None),
      ),
      right: LengthValue::from_px(
        border_delta.right.to_px(0, None)
          + shadow_border_delta.right.to_px(0, None),
      ),
      top: LengthValue::from_px(
        border_delta.top.to_px(0, None)
          + shadow_border_delta.top.to_px(0, None),
      ),
      bottom: LengthValue::from_px(
        border_delta.bottom.to_px(0, None)
          + shadow_border_delta.bottom.to_px(0, None),
      ),
    })
  }

  /// Gets whether the window should be fullscreen for the given workspace.
  ///
  /// A window is considered fullscreen if its frame covers or exceeds the
  /// workspace bounds, meaning all sides extend into the outer gaps.
  ///
  /// NOTE: The OS can be off by up to 1px when positioning windows.
  fn should_fullscreen(
    &self,
    workspace: &Workspace,
  ) -> anyhow::Result<bool> {
    let frame = self.native_properties().frame;
    let workspace_rect = workspace.max_workspace_rect()?;

    // Check if the window frame covers the workspace bounds (with 1px of
    // leeway).
    let is_covering = frame.contains_rect(&workspace_rect.inset(1));

    // A workspace with one tiling window will have that window cover the
    // workspace bounds, but it should not be considered fullscreen.
    let is_single_tiling_window = self.state() == WindowState::Tiling
      && self.tiling_siblings().count() == 0
      && workspace_rect.inset(-1).contains_rect(&frame);

    Ok(is_covering && !is_single_tiling_window)
  }

  fn display_state(&self) -> DisplayState;

  fn set_display_state(&self, display_state: DisplayState);

  fn has_pending_dpi_adjustment(&self) -> bool;

  fn set_has_pending_dpi_adjustment(
    &self,
    has_pending_dpi_adjustment: bool,
  );

  fn floating_placement(&self) -> Rect;

  fn set_floating_placement(&self, floating_placement: Rect);

  fn has_custom_floating_placement(&self) -> bool;

  fn set_has_custom_floating_placement(
    &self,
    has_custom_floating_placement: bool,
  );

  fn done_window_rules(&self) -> Vec<WindowRuleConfig>;

  fn set_done_window_rules(
    &self,
    done_window_rules: Vec<WindowRuleConfig>,
  );

  fn active_drag(&self) -> Option<ActiveDrag>;

  fn set_active_drag(&self, active_drag: Option<ActiveDrag>);

  /// Gets the cached native window properties.
  fn native_properties(&self) -> NativeWindowProperties;

  /// Updates the cached native window properties using a closure.
  fn update_native_properties<F>(&self, updater: F)
  where
    F: FnOnce(&mut NativeWindowProperties);
}

/// Implements the `WindowGetters` trait for a given struct.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with a `state`, `prev_state`, `native`, `has_pending_dpi_adjustment`,
/// `border_delta`, `display_state`, and a `done_window_rules` field.
#[macro_export]
macro_rules! impl_window_getters {
  ($struct_name:ident) => {
    impl WindowGetters for $struct_name {
      fn state(&self) -> WindowState {
        self.0.borrow().state.clone()
      }

      fn set_state(&self, state: WindowState) {
        self.0.borrow_mut().state = state;
      }

      fn prev_state(&self) -> Option<WindowState> {
        self.0.borrow().prev_state.clone()
      }

      fn set_prev_state(&self, state: WindowState) {
        self.0.borrow_mut().prev_state = Some(state);
      }

      fn native(&self) -> Ref<'_, NativeWindow> {
        Ref::map(self.0.borrow(), |inner| &inner.native)
      }

      fn border_delta(&self) -> RectDelta {
        self.0.borrow().border_delta.clone()
      }

      fn set_border_delta(&self, border_delta: RectDelta) {
        self.0.borrow_mut().border_delta = border_delta;
      }

      fn display_state(&self) -> DisplayState {
        self.0.borrow().display_state.clone()
      }

      fn set_display_state(&self, display_state: DisplayState) {
        self.0.borrow_mut().display_state = display_state;
      }

      fn has_pending_dpi_adjustment(&self) -> bool {
        self.0.borrow().has_pending_dpi_adjustment
      }

      fn set_has_pending_dpi_adjustment(
        &self,
        has_pending_dpi_adjustment: bool,
      ) {
        self.0.borrow_mut().has_pending_dpi_adjustment =
          has_pending_dpi_adjustment;
      }

      fn floating_placement(&self) -> Rect {
        self.0.borrow().floating_placement.clone()
      }

      fn set_floating_placement(&self, floating_placement: Rect) {
        self.0.borrow_mut().floating_placement = floating_placement;
      }

      fn has_custom_floating_placement(&self) -> bool {
        self.0.borrow().has_custom_floating_placement.clone()
      }

      fn set_has_custom_floating_placement(
        &self,
        has_custom_floating_placement: bool,
      ) {
        self.0.borrow_mut().has_custom_floating_placement =
          has_custom_floating_placement;
      }

      fn done_window_rules(&self) -> Vec<WindowRuleConfig> {
        self.0.borrow().done_window_rules.clone()
      }

      fn set_done_window_rules(
        &self,
        done_window_rules: Vec<WindowRuleConfig>,
      ) {
        self.0.borrow_mut().done_window_rules = done_window_rules;
      }

      fn active_drag(&self) -> Option<ActiveDrag> {
        self.0.borrow().active_drag.clone()
      }

      fn set_active_drag(&self, active_drag: Option<ActiveDrag>) {
        self.0.borrow_mut().active_drag = active_drag;
      }

      fn native_properties(&self) -> NativeWindowProperties {
        self.0.borrow().native_properties.clone()
      }

      fn update_native_properties<F>(&self, updater: F)
      where
        F: FnOnce(&mut NativeWindowProperties),
      {
        updater(&mut self.0.borrow_mut().native_properties);
      }
    }
  };
}
