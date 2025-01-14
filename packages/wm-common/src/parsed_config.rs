use serde::{Deserialize, Serialize};

use crate::{
  app_command::InvokeCommand, Color, LengthValue, OpacityValue, RectDelta,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ParsedConfig {
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
  pub display_name: Option<String>,

  /// Keybindings that will be active when the binding mode is active.
  pub keybindings: Vec<KeybindingConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct GapsConfig {
  /// Whether to scale the gaps with the DPI of the monitor.
  #[serde(default = "default_bool::<true>")]
  pub scale_with_dpi: bool,

  /// Gap between adjacent windows.
  pub inner_gap: LengthValue,

  /// Gap between windows and the screen edge.
  pub outer_gap: RectDelta,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct GeneralConfig {
  /// Config for automatically moving the cursor.
  pub cursor_jump: CursorJumpConfig,

  /// Whether to automatically focus windows underneath the cursor.
  #[serde(default = "default_bool::<false>")]
  pub focus_follows_cursor: bool,

  /// Whether to switch back and forth between the previously focused
  /// workspace when focusing the current workspace.
  #[serde(default = "default_bool::<true>")]
  pub toggle_workspace_on_refocus: bool,

  /// Commands to run when the WM has started (e.g. to run a script or
  /// launch another application).
  #[serde(default)]
  pub startup_commands: Vec<InvokeCommand>,

  /// Commands to run just before the WM is shutdown.
  #[serde(default)]
  pub shutdown_commands: Vec<InvokeCommand>,

  /// Commands to run after the WM config has reloaded.
  #[serde(default)]
  pub config_reload_commands: Vec<InvokeCommand>,

  /// How windows should be hidden when switching workspaces.
  #[serde(default)]
  pub hide_method: HideMethod,

  /// Affects which windows get shown in the native Windows taskbar.
  #[serde(default = "default_bool::<false>")]
  pub show_all_in_taskbar: bool,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HideMethod {
  Hide,
  #[default]
  Cloak,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CursorJumpConfig {
  /// Whether to automatically move the cursor on the specified trigger.
  #[serde(default = "default_bool::<true>")]
  pub enabled: bool,

  /// Trigger for cursor jump.
  #[serde(default)]
  pub trigger: CursorJumpTrigger,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CursorJumpTrigger {
  #[default]
  MonitorFocus,
  WindowFocus,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct KeybindingConfig {
  /// Keyboard shortcut to trigger the keybinding.
  pub bindings: Vec<String>,

  /// WM commands to run when the keybinding is triggered.
  pub commands: Vec<InvokeCommand>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WindowBehaviorConfig {
  /// New windows are created in this state whenever possible.
  #[serde(default)]
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WindowStateDefaultsConfig {
  pub floating: FloatingStateConfig,
  pub fullscreen: FullscreenStateConfig,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FloatingStateConfig {
  /// Whether to center new floating windows.
  #[serde(default = "default_bool::<true>")]
  pub centered: bool,

  /// Whether to show floating windows as always on top.
  #[serde(default = "default_bool::<false>")]
  pub shown_on_top: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FullscreenStateConfig {
  /// Whether to prefer fullscreen windows to be maximized.
  #[serde(default = "default_bool::<true>")]
  pub maximized: bool,

  /// Whether to show fullscreen windows as always on top.
  #[serde(default = "default_bool::<false>")]
  pub shown_on_top: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WindowEffectsConfig {
  /// Visual effects to apply to the focused window.
  pub focused_window: WindowEffectConfig,

  /// Visual effects to apply to non-focused windows.
  pub other_windows: WindowEffectConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WindowEffectConfig {
  /// Config for optionally applying a colored border.
  pub border: BorderEffectConfig,

  /// Config for optionally hiding the title bar.
  #[serde(default)]
  pub hide_title_bar: HideTitleBarEffectConfig,

  /// Config for optionally changing the corner style.
  #[serde(default)]
  pub corner_style: CornerEffectConfig,

  /// Config for optionally applying transparency.
  #[serde(default)]
  pub transparency: TransparencyEffectConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct BorderEffectConfig {
  /// Whether to enable the effect.
  #[serde(default = "default_bool::<false>")]
  pub enabled: bool,

  /// Color of the window border.
  #[serde(default = "default_blue")]
  pub color: Color,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct HideTitleBarEffectConfig {
  /// Whether to enable the effect.
  #[serde(default = "default_bool::<false>")]
  pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CornerEffectConfig {
  /// Whether to enable the effect.
  #[serde(default = "default_bool::<false>")]
  pub enabled: bool,

  /// Style of the window corners.
  #[serde(default)]
  pub style: CornerStyle,
}

#[derive(
  Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum CornerStyle {
  #[default]
  Default,
  Square,
  Rounded,
  SmallRounded,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct TransparencyEffectConfig {
  /// Whether to enable the effect.
  #[serde(default = "default_bool::<false>")]
  pub enabled: bool,

  /// The opacity to apply.
  #[serde(default)]
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
#[serde(rename_all(serialize = "camelCase"))]
pub struct WindowMatchConfig {
  #[serde(default)]
  pub window_process: Option<MatchType>,

  #[serde(default)]
  pub window_class: Option<MatchType>,

  #[serde(default)]
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
      MatchType::Regex { regex } => regex::Regex::new(regex)
        .map(|re| re.is_match(value))
        .unwrap_or(false),
      MatchType::NotEquals { not_equals } => value != not_equals,
      MatchType::NotRegex { not_regex } => regex::Regex::new(not_regex)
        .map(|re| !re.is_match(value))
        .unwrap_or(false),
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
  pub display_name: Option<String>,
  pub bind_to_monitor: Option<u32>,
  #[serde(default = "default_bool::<false>")]
  pub keep_alive: bool,
}

/// Helper function for setting a default value for a boolean field.
const fn default_bool<const V: bool>() -> bool {
  V
}

/// Helper function for setting a default value for a color field.
const fn default_blue() -> Color {
  Color {
    r: 140,
    g: 190,
    b: 255,
    a: 255,
  }
}

/// Helper function for setting a default value for window rule events.
fn default_window_rule_on() -> Vec<WindowRuleEvent> {
  vec![WindowRuleEvent::Manage, WindowRuleEvent::TitleChange]
}

impl Default for CornerEffectConfig {
  fn default() -> Self {
    CornerEffectConfig {
      enabled: false,
      style: CornerStyle::Default,
    }
  }
}
