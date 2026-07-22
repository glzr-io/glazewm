use serde::{Deserialize, Serialize};
use wm_platform::{
  Color, CornerStyle, Key, Keybinding, LengthValue, OpacityValue,
  RectDelta,
};

use crate::app_command::InvokeCommand;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct ParsedConfig {
  pub animations: AnimationsConfig,
  pub binding_modes: Vec<BindingModeConfig>,
  pub gaps: GapsConfig,
  pub general: GeneralConfig,
  pub keybindings: Vec<KeybindingConfig>,
  pub window_behavior: WindowBehaviorConfig,
  pub window_effects: WindowEffectsConfig,
  pub window_rules: Vec<WindowRuleConfig>,
  pub workspaces: Vec<WorkspaceConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct BindingModeConfig {
  /// Name of the binding mode.
  pub name: String,

  /// Display name of the binding mode.
  #[serde(default)]
  pub display_name: Option<String>,

  /// Keybindings that will be active when the binding mode is active.
  #[serde(default)]
  pub keybindings: Vec<KeybindingConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct GapsConfig {
  /// Whether to scale the gaps with the DPI of the monitor.
  pub scale_with_dpi: bool,

  /// Gap between adjacent windows.
  pub inner_gap: LengthValue,

  /// Gap between windows and the screen edge.
  pub outer_gap: RectDelta,

  /// Gap between window and the screen edge if there is only one window
  /// in the workspace
  pub single_window_outer_gap: Option<RectDelta>,
}

impl Default for GapsConfig {
  fn default() -> Self {
    GapsConfig {
      scale_with_dpi: true,
      inner_gap: LengthValue::from_px(0),
      outer_gap: RectDelta::new(
        LengthValue::from_px(0),
        LengthValue::from_px(0),
        LengthValue::from_px(0),
        LengthValue::from_px(0),
      ),
      single_window_outer_gap: None,
    }
  }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct GeneralConfig {
  /// Config for automatically moving the cursor.
  pub cursor_jump: CursorJumpConfig,

  /// Whether to automatically focus windows underneath the cursor.
  pub focus_follows_cursor: bool,

  /// Whether to switch back and forth between the previously focused
  /// workspace when focusing the current workspace.
  pub toggle_workspace_on_refocus: bool,

  /// Commands to run when the WM has started (e.g. to run a script or
  /// launch another application).
  pub startup_commands: Vec<InvokeCommand>,

  /// Commands to run just before the WM is shutdown.
  pub shutdown_commands: Vec<InvokeCommand>,

  /// Commands to run after the WM config has reloaded.
  pub config_reload_commands: Vec<InvokeCommand>,

  /// How windows should be hidden when switching workspaces.
  #[serde(deserialize_with = "deserialize_hide_method")]
  pub hide_method: HideMethod,

  /// Affects which windows get shown in the native Windows taskbar.
  pub show_all_in_taskbar: bool,
}

impl Default for GeneralConfig {
  fn default() -> Self {
    GeneralConfig {
      cursor_jump: CursorJumpConfig::default(),
      focus_follows_cursor: false,
      toggle_workspace_on_refocus: true,
      startup_commands: vec![],
      shutdown_commands: vec![],
      config_reload_commands: vec![],
      hide_method: {
        #[cfg(target_os = "macos")]
        {
          HideMethod::PlaceInCorner
        }
        #[cfg(not(target_os = "macos"))]
        {
          HideMethod::Cloak
        }
      },
      show_all_in_taskbar: false,
    }
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct CursorJumpConfig {
  /// Whether to automatically move the cursor on the specified trigger.
  pub enabled: bool,

  /// Trigger for cursor jump.
  pub trigger: CursorJumpTrigger,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CursorJumpTrigger {
  #[default]
  MonitorFocus,
  WindowFocus,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HideMethod {
  Hide,
  #[default]
  Cloak,
  PlaceInCorner,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct KeybindingConfig {
  /// Keyboard shortcut to trigger the keybinding.
  #[serde(
    deserialize_with = "deserialize_bindings",
    serialize_with = "serialize_bindings"
  )]
  pub bindings: Vec<Keybinding>,

  /// WM commands to run when the keybinding is triggered.
  pub commands: Vec<InvokeCommand>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowBehaviorConfig {
  /// New windows are created in this state whenever possible.
  pub initial_state: InitialWindowState,

  /// Sets the default options for when a new window is created. This also
  /// changes the defaults for when the state change commands, like
  /// `set_floating`, are used without any flags.
  pub state_defaults: WindowStateDefaultsConfig,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InitialWindowState {
  #[default]
  Tiling,
  Floating,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowStateDefaultsConfig {
  pub floating: FloatingStateConfig,
  pub fullscreen: FullscreenStateConfig,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct FloatingStateConfig {
  /// Whether to center new floating windows.
  pub centered: bool,

  /// Whether to show floating windows as always on top.
  pub shown_on_top: bool,
}

impl Default for FloatingStateConfig {
  fn default() -> Self {
    FloatingStateConfig {
      centered: true,
      shown_on_top: false,
    }
  }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct FullscreenStateConfig {
  /// Whether to prefer fullscreen windows to be maximized.
  pub maximized: bool,

  /// Whether to show fullscreen windows as always on top.
  pub shown_on_top: bool,
}

impl Default for FullscreenStateConfig {
  fn default() -> Self {
    FullscreenStateConfig {
      maximized: true,
      shown_on_top: false,
    }
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowEffectsConfig {
  /// Visual effects to apply to the focused window.
  pub focused_window: WindowEffectConfig,

  /// Visual effects to apply to non-focused windows.
  pub other_windows: WindowEffectConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowEffectConfig {
  /// Config for optionally applying a colored border.
  pub border: BorderEffectConfig,

  /// Config for optionally hiding the title bar.
  pub hide_title_bar: HideTitleBarEffectConfig,

  /// Config for optionally changing the corner style.
  pub corner_style: CornerEffectConfig,

  /// Config for optionally applying transparency.
  pub transparency: TransparencyEffectConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct BorderEffectConfig {
  /// Whether to enable the effect.
  pub enabled: bool,

  /// Color of the window border.
  pub color: Color,
}

impl Default for BorderEffectConfig {
  fn default() -> Self {
    BorderEffectConfig {
      enabled: false,
      color: Color {
        r: 140,
        g: 190,
        b: 255,
        a: 255,
      },
    }
  }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct HideTitleBarEffectConfig {
  /// Whether to enable the effect.
  pub enabled: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct CornerEffectConfig {
  /// Whether to enable the effect.
  pub enabled: bool,

  /// Style of the window corners.
  pub style: CornerStyle,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct TransparencyEffectConfig {
  /// Whether to enable the effect.
  pub enabled: bool,

  /// The opacity to apply.
  pub opacity: OpacityValue,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WindowRuleConfig {
  pub commands: Vec<InvokeCommand>,

  #[serde(rename = "match")]
  pub match_window: Vec<WindowMatchConfig>,

  #[serde(default = "default_window_rule_on")]
  pub on: Vec<WindowRuleEvent>,

  #[serde(default = "default_bool::<true>")]
  pub run_once: bool,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowMatchConfig {
  pub window_process: Option<MatchType>,
  pub window_class: Option<MatchType>,
  pub window_title: Option<MatchType>,
}

/// Due to limitations in `serde_yaml`, we need to use an untagged enum
/// instead of a regular enum for serialization. Using a regular enum
/// causes issues with flow-style objects in YAML.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum MatchType {
  Equals { equals: String },
  Includes { includes: String },
  Regex { regex: String },
  NotEquals { not_equals: String },
  NotRegex { not_regex: String },
}

impl MatchType {
  /// Whether the given value is a match for the match type.
  #[must_use]
  pub fn is_match(&self, value: &str) -> bool {
    match self {
      MatchType::Equals { equals } => value == equals,
      MatchType::Includes { includes } => value.contains(includes),
      MatchType::Regex { regex } => {
        regex::Regex::new(regex).is_ok_and(|re| re.is_match(value))
      }
      MatchType::NotEquals { not_equals } => value != not_equals,
      MatchType::NotRegex { not_regex } => {
        regex::Regex::new(not_regex).is_ok_and(|re| !re.is_match(value))
      }
    }
  }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowRuleEvent {
  /// When a window receives native focus.
  Focus,

  /// When a window is initially managed.
  Manage,

  /// When the title of a window changes.
  TitleChange,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WorkspaceConfig {
  pub name: String,

  #[serde(default)]
  pub display_name: Option<String>,

  #[serde(default)]
  pub bind_to_monitor: Option<u32>,

  #[serde(default = "default_bool::<false>")]
  pub keep_alive: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct AnimationsConfig {
  /// Animation settings for pure window translations (position changes
  /// only).
  pub window_move: AnimationTypeConfig,
  /// Animation settings for operations that change window size.
  pub window_resize: WindowResizeConfig,
  /// Animation settings for when a new window appears.
  ///
  /// # Platform-specific
  ///
  /// Only has an effect on Windows.
  pub window_open: WindowOpenConfig,
  /// Animation settings for workspace-switch slide transitions.
  pub workspace_switch: WorkspaceSwitchAnimationConfig,
  /// Animation settings for when a window is closed.
  ///
  /// # Platform-specific
  ///
  /// Only has an effect on Windows.
  pub window_close: WindowCloseConfig,
}

impl Default for AnimationsConfig {
  fn default() -> Self {
    AnimationsConfig {
      window_move: AnimationTypeConfig::default(),
      window_resize: WindowResizeConfig::default(),
      window_open: WindowOpenConfig::default(),
      workspace_switch: WorkspaceSwitchAnimationConfig::default(),
      window_close: WindowCloseConfig::default(),
    }
  }
}

/// Spatial style for window open/close transitions.
///
/// Used by both `WindowOpenConfig.style` and `WindowCloseConfig.style` so
/// the same values apply symmetrically: a window that opens with `slide_right`
/// (entering from the right) closes with `slide_right` (exiting to the right).
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WindowTransitionStyle {
  /// Slide in/out from/to the right edge (default).
  #[default]
  #[serde(alias = "right")]
  SlideRight,
  /// Slide in/out from/to the left edge.
  #[serde(alias = "left")]
  SlideLeft,
  /// Slide in/out from/to the top edge.
  #[serde(alias = "top")]
  SlideTop,
  /// Slide in/out from/to the bottom edge.
  #[serde(alias = "bottom")]
  SlideBottom,
  /// No positional movement. Combine with `opacity_from`/`opacity_to` for a
  /// pure fade. Accepts `"fade"` as a legacy alias.
  #[serde(alias = "fade")]
  None,
  /// Zoom in/out from the window center. Combine with `opacity_from`/`opacity_to`
  /// to also fade while zooming.
  Zoom,
}

impl WindowTransitionStyle {
  /// Returns `true` when the style has no positional slide component.
  ///
  /// Stationary styles keep the surrogate at the window's final position for
  /// the full animation; the surrogate window itself never moves.
  pub fn is_stationary(&self) -> bool {
    matches!(self, Self::None | Self::Zoom)
  }
}

/// Animation settings for when a new window appears (Windows only).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowOpenConfig {
  pub enabled: bool,
  pub duration_ms: u32,
  pub easing: EasingFunction,
  /// Transition type for the open animation.
  ///
  /// - `slide_right` (default): slides in from the right.
  /// - `slide_left` / `slide_top` / `slide_bottom`: slide from that edge.
  /// - `none` / `fade`: no slide; combine with `opacity_from` for a pure
  ///   fade-in.
  /// - `zoom`: zoom in from the window center.
  pub style: WindowTransitionStyle,
  /// Starting opacity (0.0–1.0). At `1.0` no fade is applied; at `0.0` the
  /// window fades in from fully transparent. Can be combined with any style.
  pub opacity_from: f32,
}

impl Default for WindowOpenConfig {
  fn default() -> Self {
    WindowOpenConfig {
      enabled: true,
      duration_ms: 150,
      easing: EasingFunction::CubicBezier(0.16, 1.0, 0.3, 1.0),
      style: WindowTransitionStyle::SlideRight,
      opacity_from: 1.0,
    }
  }
}

/// Animation settings for when a window is closed.
///
/// # Platform-specific
///
/// Only has an effect on Windows.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowCloseConfig {
  pub enabled: bool,
  pub duration_ms: u32,
  pub easing: EasingFunction,
  /// Transition type for the close animation.
  ///
  /// - `none` / `fade` (default): no positional movement; combine with
  ///   `opacity_to` for a pure fade-out.
  /// - `zoom`: zoom out from the window center.
  /// - `slide_right` / `slide_left` / `slide_top` / `slide_bottom`: slide off
  ///   that edge.
  pub style: WindowTransitionStyle,
  /// Final opacity (0.0–1.0). At `0.0` the window fades to fully transparent;
  /// at `1.0` no fade is applied.
  pub opacity_to: f32,
}

impl Default for WindowCloseConfig {
  fn default() -> Self {
    WindowCloseConfig {
      enabled: false,
      duration_ms: 150,
      easing: EasingFunction::CubicBezier(0.32, 0.0, 0.67, 0.0),
      style: WindowTransitionStyle::None,
      opacity_to: 0.0,
    }
  }
}

/// Motion style of the workspace-switch transition.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceSwitchStyle {
  /// Workspaces slide along the axis set by `direction` (default).
  #[default]
  Slide,
  /// Pure crossfade; no positional slide. Both surrogates stay in place and
  /// their opacities are driven by `opacity_outgoing` / `opacity_incoming`.
  Fade,
  /// Outgoing workspace shrinks to the monitor center; incoming expands from
  /// it. Opacities are also animated via `opacity_outgoing` / `opacity_incoming`.
  Zoom,
  /// Iris wipe: a frozen snapshot of the outgoing workspace stays on top while
  /// a circular hole grows from `iris_origin`, revealing the live incoming
  /// workspace beneath. Requires Windows; falls back to an instant switch when
  /// the monitor cannot be captured.
  Iris,
}

impl WorkspaceSwitchStyle {
  /// Returns `true` when the transition has no positional slide component.
  pub fn is_no_slide(&self) -> bool {
    matches!(self, Self::Fade | Self::Zoom)
  }

  /// Returns `true` for the iris-wipe style, which is driven by a single
  /// snapshot overlay rather than per-window surrogates.
  pub fn is_iris(&self) -> bool {
    matches!(self, Self::Iris)
  }
}

/// Origin point from which the iris-wipe circle grows.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceSwitchIrisOrigin {
  /// Grow from the center of the monitor (default).
  #[default]
  Center,
  /// Grow from the current mouse-cursor position.
  Cursor,
  /// Grow from the center of the newly focused window on the incoming
  /// workspace. Falls back to the monitor center when the incoming workspace
  /// has no focusable window.
  FocusedWindow,
}

/// Slide axis for the `slide` workspace-switch style.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceSwitchDirection {
  /// Slide left/right (default).
  #[default]
  Horizontal,
  /// Slide up/down.
  Vertical,
}

/// Animation config for workspace-switch transitions.
///
/// Outgoing workspaces translate off-screen (for the `slide` style) or stay in
/// place (for `fade`/`zoom`) while the incoming workspace slides or crossfades
/// in, all constrained to the monitor on which the switch occurs.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WorkspaceSwitchAnimationConfig {
  pub enabled: bool,
  pub duration_ms: u32,
  pub easing: EasingFunction,
  /// Motion type: `slide` (default), `fade`, `zoom`, or `iris`.
  pub style: WorkspaceSwitchStyle,
  /// Slide axis when `style` is `slide`: `horizontal` (default) or `vertical`.
  pub direction: WorkspaceSwitchDirection,
  /// Origin of the iris circle when `style` is `iris`: `center` (default),
  /// `cursor`, or `focused_window`. Ignored by other styles.
  pub iris_origin: WorkspaceSwitchIrisOrigin,
  /// Opacity at the end of the outgoing workspace's animation (0.0–1.0).
  ///
  /// At `1.0` (default) the outgoing workspace stays fully opaque. At `0.0` it
  /// fades out to transparent. Any value in between produces a partial fade.
  /// Applies to all `style` values.
  pub opacity_outgoing: f32,
  /// Opacity at the start of the incoming workspace's animation (0.0–1.0).
  ///
  /// At `1.0` (default) the incoming workspace starts fully opaque. At `0.0`
  /// it fades in from transparent. Any value in between produces a partial
  /// fade. Applies to all `style` values.
  pub opacity_incoming: f32,
  /// Amount of workspace-level scale applied during `slide` transitions.
  ///
  /// The outgoing workspace shrinks from `1.0` to `1.0 - zoom_factor` as it
  /// exits; the incoming grows from `1.0 - zoom_factor` to `1.0` as it enters.
  /// Scaling is from the monitor center so all windows move inward together,
  /// preserving the workspace-as-a-panel illusion. Has no effect on `fade` or
  /// `zoom` styles. Valid range: `0.0` (no zoom) to `1.0` (collapses to a
  /// point). Recommended range: `0.05`–`0.15` for a subtle depth effect.
  pub zoom_factor: f32,
}

impl Default for WorkspaceSwitchAnimationConfig {
  fn default() -> Self {
    WorkspaceSwitchAnimationConfig {
      enabled: true,
      duration_ms: 250,
      easing: EasingFunction::CubicBezier(0.16, 1.0, 0.3, 1.0),
      style: WorkspaceSwitchStyle::default(),
      direction: WorkspaceSwitchDirection::default(),
      iris_origin: WorkspaceSwitchIrisOrigin::default(),
      opacity_outgoing: 1.0,
      opacity_incoming: 1.0,
      zoom_factor: 0.1,
    }
  }
}

/// Animation settings for window move operations.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct AnimationTypeConfig {
  pub enabled: bool,
  pub duration_ms: u32,
  pub easing: EasingFunction,
  /// Minimum pixel distance required to trigger movement animations.
  /// Helps prevent animations from starting on very small position
  /// changes. Increase this value on high-DPI displays to reduce
  /// sensitivity.
  pub threshold_px: u32,
}

impl Default for AnimationTypeConfig {
  fn default() -> Self {
    AnimationTypeConfig {
      enabled: true,
      duration_ms: 150,
      easing: EasingFunction::CubicBezier(0.42, 0.0, 0.58, 1.0),
      threshold_px: 10,
    }
  }
}

/// Animation settings for window resize operations.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
pub struct WindowResizeConfig {
  pub enabled: bool,
  pub duration_ms: u32,
  pub easing: EasingFunction,
  /// Minimum pixel distance required to trigger resize animations.
  /// Increase this value on high-DPI displays to reduce sensitivity.
  pub threshold_px: u32,
}

impl Default for WindowResizeConfig {
  fn default() -> Self {
    WindowResizeConfig {
      enabled: true,
      duration_ms: 150,
      easing: EasingFunction::CubicBezier(0.42, 0.0, 0.58, 1.0),
      threshold_px: 10,
    }
  }
}

/// Easing function for animations.
///
/// Named aliases map to their CSS cubic-bezier equivalents and can be used
/// interchangeably with `cubic_bezier(x1, y1, x2, y2)` notation:
/// `linear`, `ease_in`, `ease_out`, `ease_in_out`,
/// `ease_in_cubic`, `ease_out_cubic`, `ease_in_out_cubic`, `ease_out_spring`.
#[derive(Clone, Debug, PartialEq)]
pub enum EasingFunction {
  /// CSS cubic bezier curve: `cubic_bezier(x1, y1, x2, y2)`.
  ///
  /// Control points `(x1, y1)` and `(x2, y2)` define the shape between the
  /// implicit anchors `(0, 0)` and `(1, 1)`. `x1` and `x2` must be in
  /// `[0, 1]`; `y1` and `y2` may exceed that range to produce overshoot.
  CubicBezier(f32, f32, f32, f32),
  /// Exponentially-decaying spring. Overshoots past 1.0 and oscillates before
  /// settling. Runs to full wall-clock duration to preserve the bounce.
  EaseOutSpring,
}

impl Default for EasingFunction {
  fn default() -> Self {
    EasingFunction::CubicBezier(0.42, 0.0, 0.58, 1.0) // ease_in_out
  }
}

impl Eq for EasingFunction {}

impl EasingFunction {
  /// Returns `true` when this function can produce values outside `[0, 1]`.
  ///
  /// Non-overshooting functions are cut off at 99% eased progress to avoid
  /// the "stuck at destination" look. Overshooting ones run to full wall-clock
  /// duration to preserve their bounce.
  pub fn can_overshoot(&self) -> bool {
    match self {
      EasingFunction::EaseOutSpring => true,
      EasingFunction::CubicBezier(_, y1, _, y2) => {
        *y1 < 0.0 || *y1 > 1.0 || *y2 < 0.0 || *y2 > 1.0
      }
    }
  }
}

impl<'de> Deserialize<'de> for EasingFunction {
  fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
    let s = String::deserialize(d)?;
    // Named aliases expand to their CSS cubic-bezier control points.
    match s.as_str() {
      "linear" => Ok(EasingFunction::CubicBezier(0.0, 0.0, 1.0, 1.0)),
      "ease_in_out" => Ok(EasingFunction::CubicBezier(0.42, 0.0, 0.58, 1.0)),
      "ease_in" => Ok(EasingFunction::CubicBezier(0.42, 0.0, 1.0, 1.0)),
      "ease_out" => Ok(EasingFunction::CubicBezier(0.0, 0.0, 0.58, 1.0)),
      "ease_in_out_cubic" => Ok(EasingFunction::CubicBezier(0.65, 0.0, 0.35, 1.0)),
      "ease_in_cubic" => Ok(EasingFunction::CubicBezier(0.32, 0.0, 0.67, 0.0)),
      "ease_out_cubic" => Ok(EasingFunction::CubicBezier(0.33, 1.0, 0.68, 1.0)),
      "ease_out_spring" => Ok(EasingFunction::EaseOutSpring),
      s => {
        if let Some(inner) = s
          .strip_prefix("cubic_bezier(")
          .and_then(|s| s.strip_suffix(')'))
        {
          let parts: Vec<&str> = inner.split(',').collect();
          if parts.len() != 4 {
            return Err(serde::de::Error::custom(
              "cubic_bezier requires exactly 4 arguments: \
               cubic_bezier(x1, y1, x2, y2)",
            ));
          }
          let mut floats = [0f32; 4];
          for (i, part) in parts.iter().enumerate() {
            floats[i] = part.trim().parse::<f32>().map_err(|_| {
              serde::de::Error::custom(format!(
                "cubic_bezier argument {} is not a valid number: {}",
                i + 1,
                part.trim()
              ))
            })?;
          }
          let [x1, y1, x2, y2] = floats;
          if !(0.0..=1.0).contains(&x1) || !(0.0..=1.0).contains(&x2) {
            return Err(serde::de::Error::custom(
              "cubic_bezier x1 and x2 must be in [0, 1]",
            ));
          }
          Ok(EasingFunction::CubicBezier(x1, y1, x2, y2))
        } else {
          Err(serde::de::Error::custom(format!(
            "unknown easing function '{s}'; valid values: linear, \
             ease_in, ease_out, ease_in_out, ease_in_cubic, \
             ease_out_cubic, ease_in_out_cubic, ease_out_spring, \
             cubic_bezier(x1, y1, x2, y2)"
          )))
        }
      }
    }
  }
}

impl Serialize for EasingFunction {
  fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
    match self {
      EasingFunction::EaseOutSpring => s.serialize_str("ease_out_spring"),
      EasingFunction::CubicBezier(x1, y1, x2, y2) => {
        // Serialize back to a named alias when the control points match exactly,
        // so round-tripped configs stay human-readable.
        let repr = if *x1 == 0.0 && *y1 == 0.0 && *x2 == 1.0 && *y2 == 1.0 {
          "linear".to_string()
        } else if *x1 == 0.42 && *y1 == 0.0 && *x2 == 0.58 && *y2 == 1.0 {
          "ease_in_out".to_string()
        } else if *x1 == 0.42 && *y1 == 0.0 && *x2 == 1.0 && *y2 == 1.0 {
          "ease_in".to_string()
        } else if *x1 == 0.0 && *y1 == 0.0 && *x2 == 0.58 && *y2 == 1.0 {
          "ease_out".to_string()
        } else if *x1 == 0.65 && *y1 == 0.0 && *x2 == 0.35 && *y2 == 1.0 {
          "ease_in_out_cubic".to_string()
        } else if *x1 == 0.32 && *y1 == 0.0 && *x2 == 0.67 && *y2 == 0.0 {
          "ease_in_cubic".to_string()
        } else if *x1 == 0.33 && *y1 == 1.0 && *x2 == 0.68 && *y2 == 1.0 {
          "ease_out_cubic".to_string()
        } else {
          format!("cubic_bezier({x1}, {y1}, {x2}, {y2})")
        };
        s.serialize_str(&repr)
      }
    }
  }
}

/// Helper function for setting a default value for a boolean field.
const fn default_bool<const V: bool>() -> bool {
  V
}

/// Helper function for setting a default value for window rule events.
fn default_window_rule_on() -> Vec<WindowRuleEvent> {
  vec![WindowRuleEvent::Manage, WindowRuleEvent::TitleChange]
}

/// Helper function for serializing a vector of keybindings.
///
/// Returns a vector of strings (e.g. `["cmd+shift+a", "ctrl+shift+b"]`).
fn serialize_bindings<S>(
  bindings: &[Keybinding],
  serializer: S,
) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer,
{
  let binding_strings: Vec<String> = bindings
    .iter()
    .map(|binding| {
      binding
        .keys()
        .iter()
        .map(|key| key.to_string().to_lowercase())
        .collect::<Vec<_>>()
        .join("+")
    })
    .collect();

  binding_strings.serialize(serializer)
}

/// Helper function for deserializing a vector of strings into keybindings.
///
/// Returns a vector of [`Keybinding`].
fn deserialize_bindings<'de, D>(
  deserializer: D,
) -> Result<Vec<Keybinding>, D::Error>
where
  D: serde::de::Deserializer<'de>,
{
  let s: Vec<&str> = serde::de::Deserialize::deserialize(deserializer)?;
  s.iter()
    .map(|keybinding_str| {
      let keys: Vec<Key> = keybinding_str
        .split('+')
        .map(|key| {
          key.trim().parse().or_else(|_| Key::try_from_literal(key))
        })
        .collect::<Result<Vec<Key>, _>>()
        .map_err(serde::de::Error::custom)?;

      Keybinding::new(keys).map_err(serde::de::Error::custom)
    })
    .collect()
}

/// Helper function for deserializing [`HideMethod`].
///
/// On macOS, [`HideMethod::Hide`] and [`HideMethod::Cloak`] are not valid
/// and are automatically converted to [`HideMethod::PlaceInCorner`].
fn deserialize_hide_method<'de, D>(
  deserializer: D,
) -> Result<HideMethod, D::Error>
where
  D: serde::de::Deserializer<'de>,
{
  // LINT: The deserialized value is ignored on macOS, but we still want
  // to produce an error for invalid values.
  #[allow(unused_variables)]
  let method = HideMethod::deserialize(deserializer)?;

  #[cfg(target_os = "macos")]
  {
    Ok(HideMethod::PlaceInCorner)
  }

  #[cfg(not(target_os = "macos"))]
  {
    Ok(method)
  }
}
