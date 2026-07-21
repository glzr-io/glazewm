use serde::{Deserialize, Deserializer, Serialize};
use wm_platform::{
  Color, CornerStyle, Key, Keybinding, LengthValue, OpacityValue,
  RectDelta,
};

use crate::app_command::{ColumnBias, InvokeCommand};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all(serialize = "camelCase"))]
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

  /// A default column layout applied to any workspace that has no
  /// `columns:` of its own, chosen from the shape of the monitor the
  /// workspace currently occupies. See [`DefaultColumns`].
  #[serde(default)]
  pub default_columns: Option<DefaultColumns>,
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
      default_columns: None,
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

  /// A column layout assigned to this workspace, applied whenever the
  /// workspace is focused. Accepts either a bare spec string
  /// (`columns: "*,C,*"`) or a full object (`columns: { spec, center,
  /// bias }`).
  #[serde(default)]
  pub columns: Option<ColumnLayout>,
}

/// A `columns` layout assigned to a workspace. Applied on focus and
/// reapplied on every switch back to the workspace.
///
/// Deserializes from either a bare spec string (`"*,C,*"`) or a full
/// object (`{ spec: "*,C,*", center: 0.6, bias: left }`).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct ColumnLayout {
  pub spec: String,
  pub center: f32,
  pub bias: ColumnBias,
}

/// Helper function for the default center width of a column layout.
fn default_columns_center() -> f32 {
  0.6
}

impl<'de> Deserialize<'de> for ColumnLayout {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
      Spec(String),
      Full {
        spec: String,
        #[serde(default = "default_columns_center")]
        center: f32,
        #[serde(default)]
        bias: ColumnBias,
      },
    }

    Ok(match Raw::deserialize(deserializer)? {
      Raw::Spec(spec) => ColumnLayout {
        spec,
        center: default_columns_center(),
        bias: ColumnBias::default(),
      },
      Raw::Full { spec, center, bias } => {
        ColumnLayout { spec, center, bias }
      }
    })
  }
}

/// A monitor-shape-gated default column layout, applied to any workspace
/// that has no `columns:` of its own. The layout is the first rule whose
/// aspect-ratio band contains the workspace's current monitor's aspect
/// ratio; a workspace matching no rule keeps the default tiling.
///
/// Deserializes from a bare spec string (`"*,C,*"`), a single rule object,
/// or an ordered list of rule objects (first match wins).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct DefaultColumns {
  pub rules: Vec<DefaultColumnsRule>,
}

/// One aspect-ratio-gated rule within a [`DefaultColumns`].
///
/// `min_aspect_ratio` (inclusive) and `max_aspect_ratio` (exclusive) bound
/// the monitor aspect ratios (`width / height`, so landscape > 1) the rule
/// matches; either or both may be omitted for an open-ended or catch-all
/// rule. `columns` is the layout applied when the rule matches, or `None`
/// to explicitly keep default tiling for the band (spec `default`/`none`).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct DefaultColumnsRule {
  pub min_aspect_ratio: Option<f32>,
  pub max_aspect_ratio: Option<f32>,
  pub columns: Option<ColumnLayout>,
}

impl DefaultColumns {
  /// The columns to apply on a monitor of the given `aspect_ratio`
  /// (`width / height`): the first rule whose band contains it, resolved
  /// to its columns.
  ///
  /// Returns `None` when no rule matches, or the matched rule is an
  /// explicit `default`/`none` — both meaning "keep default tiling". A
  /// rule matches when `aspect_ratio` is `>= min_aspect_ratio` (when set)
  /// and `< max_aspect_ratio` (when set).
  #[must_use]
  pub fn columns_for(&self, aspect_ratio: f32) -> Option<ColumnLayout> {
    self
      .rules
      .iter()
      .find(|rule| {
        rule.min_aspect_ratio.is_none_or(|min| aspect_ratio >= min)
          && rule.max_aspect_ratio.is_none_or(|max| aspect_ratio < max)
      })
      .and_then(|rule| rule.columns.clone())
  }
}

impl<'de> Deserialize<'de> for DefaultColumns {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    /// A rule as written in config, with the spec/center/bias flattened
    /// alongside the aspect-ratio bounds.
    #[derive(Deserialize)]
    struct RawRule {
      #[serde(default)]
      min_aspect_ratio: Option<f32>,
      #[serde(default)]
      max_aspect_ratio: Option<f32>,
      spec: String,
      #[serde(default = "default_columns_center")]
      center: f32,
      #[serde(default)]
      bias: ColumnBias,
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
      Spec(String),
      Rules(Vec<RawRule>),
      Single(RawRule),
    }

    /// Converts a raw rule into a validated [`DefaultColumnsRule`],
    /// mapping the `default`/`none` spec sentinel to no columns.
    fn build_rule<E: serde::de::Error>(
      raw: RawRule,
    ) -> Result<DefaultColumnsRule, E> {
      if let (Some(min), Some(max)) =
        (raw.min_aspect_ratio, raw.max_aspect_ratio)
      {
        if min >= max {
          return Err(E::custom(format!(
            "`min_aspect_ratio` ({min}) must be less than \
             `max_aspect_ratio` ({max})."
          )));
        }
      }

      for bound in [raw.min_aspect_ratio, raw.max_aspect_ratio]
        .into_iter()
        .flatten()
      {
        if bound <= 0.0 {
          return Err(E::custom(
            "Aspect-ratio bounds must be greater than 0.",
          ));
        }
      }

      let columns = if matches!(
        raw.spec.to_lowercase().as_str(),
        "default" | "none"
      ) {
        None
      } else {
        Some(ColumnLayout {
          spec: raw.spec,
          center: raw.center,
          bias: raw.bias,
        })
      };

      Ok(DefaultColumnsRule {
        min_aspect_ratio: raw.min_aspect_ratio,
        max_aspect_ratio: raw.max_aspect_ratio,
        columns,
      })
    }

    let raw_rules = match Raw::deserialize(deserializer)? {
      Raw::Spec(spec) => vec![RawRule {
        min_aspect_ratio: None,
        max_aspect_ratio: None,
        spec,
        center: default_columns_center(),
        bias: ColumnBias::default(),
      }],
      Raw::Single(rule) => vec![rule],
      Raw::Rules(rules) => rules,
    };

    let rules = raw_rules
      .into_iter()
      .map(build_rule)
      .collect::<Result<Vec<_>, D::Error>>()?;

    Ok(DefaultColumns { rules })
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

#[cfg(test)]
mod tests {
  use super::{ColumnBias, DefaultColumns};

  fn parse(yaml: &str) -> DefaultColumns {
    serde_yaml::from_str(yaml).expect("valid default_columns")
  }

  #[test]
  fn deserializes_bare_spec_string() {
    let parsed = parse(r#""*,C,*""#);

    assert_eq!(parsed.rules.len(), 1);
    let rule = &parsed.rules[0];
    assert_eq!(rule.min_aspect_ratio, None);
    assert_eq!(rule.max_aspect_ratio, None);
    let assignment = rule.columns.as_ref().expect("columns");
    assert_eq!(assignment.spec, "*,C,*");
    assert_eq!(assignment.bias, ColumnBias::Left);
  }

  #[test]
  fn deserializes_single_object() {
    let parsed = parse("{ spec: \"C,*\", center: 0.5, bias: right }");

    assert_eq!(parsed.rules.len(), 1);
    let assignment = parsed.rules[0].columns.as_ref().expect("columns");
    assert_eq!(assignment.spec, "C,*");
    assert!((assignment.center - 0.5).abs() < f32::EPSILON);
    assert_eq!(assignment.bias, ColumnBias::Right);
  }

  #[test]
  fn deserializes_rule_list_and_default_sentinel() {
    let parsed = parse(
      "
      - min_aspect_ratio: 2.0
        spec: \"*,C,*\"
      - min_aspect_ratio: 1.4
        spec: default
      - spec: \"C\"
      ",
    );

    assert_eq!(parsed.rules.len(), 3);
    assert_eq!(parsed.rules[0].min_aspect_ratio, Some(2.0));
    assert!(parsed.rules[0].columns.is_some());
    // `default` is an explicit "keep default tiling" band.
    assert!(parsed.rules[1].columns.is_none());
    assert!(parsed.rules[2].columns.is_some());
  }

  #[test]
  fn rejects_inverted_bounds() {
    let result: Result<DefaultColumns, _> = serde_yaml::from_str(
      "
      - min_aspect_ratio: 2.0
        max_aspect_ratio: 1.0
        spec: \"C\"
      ",
    );

    assert!(result.is_err());
  }

  #[test]
  fn columns_for_picks_first_matching_band() {
    let parsed = parse(
      "
      - min_aspect_ratio: 2.0
        spec: ultrawide
      - min_aspect_ratio: 1.4
        spec: wide
      ",
    );

    // Ultrawide → first rule.
    assert_eq!(
      parsed.columns_for(2.37).map(|s| s.spec),
      Some("ultrawide".to_string())
    );
    // ~16:9 → second rule (first doesn't match).
    assert_eq!(
      parsed.columns_for(1.78).map(|s| s.spec),
      Some("wide".to_string())
    );
    // 4:3 → no rule matches → default tiling.
    assert_eq!(parsed.columns_for(1.33).map(|s| s.spec), None);
  }

  #[test]
  fn columns_for_bounds_are_min_inclusive_max_exclusive() {
    let parsed = parse(
      "
      - min_aspect_ratio: 1.5
        max_aspect_ratio: 2.0
        spec: band
      ",
    );

    assert!(parsed.columns_for(1.5).is_some(), "min is inclusive");
    assert!(parsed.columns_for(1.99).is_some(), "inside band");
    assert!(parsed.columns_for(2.0).is_none(), "max is exclusive");
    assert!(parsed.columns_for(1.49).is_none(), "below band");
  }

  #[test]
  fn columns_for_returns_none_on_matched_default_band() {
    let parsed = parse(
      "
      - min_aspect_ratio: 1.4
        spec: none
      ",
    );

    // The band matches, but resolves to default tiling.
    assert!(parsed.columns_for(1.78).is_none());
  }
}
