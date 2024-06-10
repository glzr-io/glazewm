use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize};
use tokio::fs;

use crate::{
  app_command::InvokeCommand,
  common::{Color, LengthValue, RectDelta},
  monitors::Monitor,
  workspaces::Workspace,
};

const SAMPLE_CONFIG: &str =
  include_str!("../../../resources/sample-config.yaml");

#[derive(Debug)]
pub struct UserConfig {
  pub path: PathBuf,
  pub value: ParsedConfig,
  pub value_str: String,
}

impl UserConfig {
  /// Creates an instance of `UserConfig`. Reads and validates the user
  /// config from the given path.
  ///
  /// Creates a new config file from sample if it doesn't exist.
  pub async fn new(config_path: Option<PathBuf>) -> anyhow::Result<Self> {
    let default_config_path = home::home_dir()
      .context("Unable to get home directory.")?
      .join(".glzr/glazewm/config.yaml");

    let config_path = config_path.unwrap_or(default_config_path);

    let (config_value, config_str) = Self::read(&config_path).await?;

    Ok(Self {
      path: config_path,
      value: config_value,
      value_str: config_str,
    })
  }

  /// Reads and validates the user config from the given path.
  ///
  /// Creates a new config file from sample if it doesn't exist.
  async fn read(
    config_path: &PathBuf,
  ) -> anyhow::Result<(ParsedConfig, String)> {
    if !config_path.exists() {
      Self::create_sample(config_path.clone()).await?;
    }

    let config_str = fs::read_to_string(&config_path)
      .await
      .context("Unable to read config file.")?;

    // TODO: Improve error formatting of serde_yaml errors. Something
    // similar to https://github.com/AlexanderThaller/format_serde_error
    let config_value = serde_yaml::from_str(&config_str)?;

    Ok((config_value, config_str))
  }

  /// Initializes a new config file from the sample config resource.
  async fn create_sample(config_path: PathBuf) -> Result<()> {
    let parent_dir =
      config_path.parent().context("Invalid config path.")?;

    fs::create_dir_all(parent_dir).await.with_context(|| {
      format!("Unable to create directory {}.", &config_path.display())
    })?;

    fs::write(&config_path, SAMPLE_CONFIG)
      .await
      .with_context(|| {
        format!("Unable to write to {}.", config_path.display())
      })?;

    Ok(())
  }

  pub async fn reload(&mut self) -> anyhow::Result<()> {
    let (config_value, config_str) = Self::read(&self.path).await?;
    self.value = config_value;
    self.value_str = config_str;

    Ok(())
  }

  pub fn inactive_workspace_configs(
    &self,
    active_workspaces: &Vec<Workspace>,
  ) -> Vec<&WorkspaceConfig> {
    self
      .value
      .workspaces
      .iter()
      .filter(|config| {
        active_workspaces
          .iter()
          .find(|workspace| workspace.config().name == config.name)
          .is_none()
      })
      .collect()
  }

  pub fn workspace_config_for_monitor(
    &self,
    monitor: &Monitor,
    active_workspaces: &Vec<Workspace>,
  ) -> Option<&WorkspaceConfig> {
    let inactive_configs =
      self.inactive_workspace_configs(active_workspaces);

    let bound_config = inactive_configs.iter().find(|&config| {
      config
        .bind_to_monitor
        .as_ref()
        .and_then(|monitor_name| {
          monitor.name().map(|n| &n == monitor_name).ok()
        })
        .unwrap_or(false)
    });

    // Get the first workspace config that isn't bound to a monitor.
    bound_config
      .or(
        inactive_configs
          .iter()
          .find(|config| config.bind_to_monitor.is_none()),
      )
      .or(inactive_configs.first())
      .cloned()
  }
}

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
  /// Gap between adjacent windows.
  #[serde(deserialize_with = "deserialize_length_value")]
  pub inner_gap: LengthValue,

  /// Gap between windows and the screen edge.
  #[serde(deserialize_with = "deserialize_rect_delta")]
  pub outer_gap: RectDelta,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct GeneralConfig {
  /// Center the cursor in the middle of a newly focused window.
  #[serde(default = "default_bool::<false>")]
  pub cursor_follows_focus: bool,

  /// Focus the window directly under the cursor at all times.
  #[serde(default = "default_bool::<false>")]
  pub focus_follows_cursor: bool,

  /// If activated, by switching to the current workspace the previous
  /// focused workspace is activated.
  #[serde(default = "default_bool::<true>")]
  pub toggle_workspace_on_refocus: bool,

  /// WM commands to run once on startup.
  #[serde(default)]
  pub startup_commands: Vec<InvokeCommand>,
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
  pub initial_state: InitialWindowState,

  /// Sets the default options for when a new window is created. This also
  /// changes the defaults for when the state change commands, like
  /// `set_floating`, are used without any flags.
  pub state_defaults: WindowStateDefaultsConfig,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InitialWindowState {
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
  pub show_on_top: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct FullscreenStateConfig {
  /// Whether to prefer fullscreen windows to be maximized.
  #[serde(default = "default_bool::<true>")]
  pub maximized: bool,

  /// Whether to show fullscreen windows as always on top.
  #[serde(default = "default_bool::<false>")]
  pub show_on_top: bool,

  /// Whether to remove the window's title bar when fullscreen.
  #[serde(default = "default_bool::<false>")]
  pub remove_title_bar: bool,
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
  /// Optional colored border to apply.
  #[serde(deserialize_with = "deserialize_option_color")]
  pub border_color: Option<Color>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WindowRuleConfig {
  pub match_process_name: Option<String>,
  pub match_class_name: Option<String>,
  pub match_title: Option<String>,
  pub commands: Vec<InvokeCommand>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct WorkspaceConfig {
  pub name: String,
  pub display_name: Option<String>,
  pub bind_to_monitor: Option<String>,
  #[serde(default = "default_bool::<false>")]
  pub keep_alive: bool,
}

/// Helper function for setting a default value for a boolean field.
const fn default_bool<const V: bool>() -> bool {
  V
}

/// Deserializes `Option<Color>` from an optional string.
fn deserialize_option_color<'de, D>(
  deserializer: D,
) -> Result<Option<Color>, D::Error>
where
  D: Deserializer<'de>,
{
  match Option::<String>::deserialize(deserializer)? {
    Some(str) => Color::from_str(&str)
      .map(Some)
      .map_err(serde::de::Error::custom),
    None => Ok(None),
  }
}

/// Deserializes `RectDelta` from a string.
fn deserialize_rect_delta<'de, D>(
  deserializer: D,
) -> Result<RectDelta, D::Error>
where
  D: Deserializer<'de>,
{
  let str = String::deserialize(deserializer)?;
  RectDelta::from_str(&str).map_err(serde::de::Error::custom)
}

/// Deserializes `LengthValue` from a string.
fn deserialize_length_value<'de, D>(
  deserializer: D,
) -> Result<LengthValue, D::Error>
where
  D: Deserializer<'de>,
{
  let str = String::deserialize(deserializer)?;
  LengthValue::from_str(&str).map_err(serde::de::Error::custom)
}
