use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer};
use tokio::fs;

use crate::{
  common::{LengthValue, RectDelta},
  wm_command::WmCommand,
};

#[derive(Debug, Deserialize)]
pub struct KeybindingConfig {
  /// Keyboard shortcut to trigger the keybinding.
  pub bindings: Vec<String>,

  /// WM commands to run when the keybinding is triggered.
  pub commands: Vec<WmCommand>,
}

#[derive(Debug, Deserialize)]
pub struct BindingModeConfig {
  /// Name of the binding mode.
  pub name: String,

  /// Keybindings that will be active when the binding mode is active.
  pub keybindings: Vec<KeybindingConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GapsConfig {
  /// Gap between adjacent windows.
  #[serde(deserialize_with = "to_length_value")]
  pub inner_gap: LengthValue,

  /// Gap between windows and the screen edge.
  #[serde(deserialize_with = "to_rect_delta")]
  pub outer_gap: RectDelta,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
  /// Whether to show floating windows as always on top.
  pub show_floating_on_top: bool,

  /// Center the cursor in the middle of a newly focused window.
  pub cursor_follows_focus: bool,

  /// Focus the window directly under the cursor at all times.
  pub focus_follows_cursor: bool,

  /// Amount by which to move floating windows
  #[serde(deserialize_with = "to_length_value")]
  pub floating_window_move_amount: LengthValue,

  /// If activated, by switching to the current workspace the previous
  /// focused workspace is activated.
  pub toggle_workspace_on_refocus: bool,

  /// Whether to enable window transition animations (on minimize, close).
  pub window_animations: WindowAnimations,

  /// Whether to center new floating windows
  pub center_new_floating_windows: bool,
}

#[derive(Debug, Deserialize)]
pub enum WindowAnimations {
  Enabled,
  Disabled,
  Unchanged,
}

#[derive(Debug, Deserialize)]
pub struct FocusBorder {
  /// Whether the default transparent border be used.
  pub enabled: bool,

  /// Border color of the window.
  pub color: String,
}

#[derive(Debug, Deserialize)]
pub struct FocusBordersConfig {
  /// Border of the focused window.
  pub active: FocusBorder,

  /// Border of non-focused windows.
  pub inactive: FocusBorder,
}

#[derive(Debug, Deserialize)]
pub struct WindowRuleConfig {
  pub match_process_name: Option<String>,
  pub match_class_name: Option<String>,
  pub match_title: Option<String>,
  pub commands: Vec<WmCommand>,
}

#[derive(Debug, Deserialize)]
pub struct WorkspaceConfig {
  pub name: String,
  pub display_name: Option<String>,
  pub bind_to_monitor: Option<String>,
  pub keep_alive: bool,
}

#[derive(Debug, Deserialize)]
pub struct UserConfig {
  pub binding_modes: Vec<BindingModeConfig>,
  pub focus_borders: FocusBordersConfig,
  pub gaps: GapsConfig,
  pub general: GeneralConfig,
  pub keybindings: Vec<KeybindingConfig>,
  pub window_rules: Vec<WindowRuleConfig>,
  pub workspaces: Vec<WorkspaceConfig>,
}

const SAMPLE_CONFIG: &str =
  include_str!("../../../resources/sample-config.yaml");

impl UserConfig {
  pub async fn read(config_path: Option<String>) -> Result<Self> {
    let default_config_path = home::home_dir()
      .context("Unable to get home directory.")?
      .join(".glzr/glazewm/config.yaml");

    let config_path = match config_path {
      Some(val) => PathBuf::from(val),
      None => default_config_path,
    };

    // Create new config file from sample if it doesn't exist.
    if !config_path.exists() {
      fs::create_dir_all(&config_path).await.with_context(|| {
        format!("Unable to create directory {}.", &config_path.display())
      })?;

      fs::write(&config_path, SAMPLE_CONFIG)
        .await
        .with_context(|| {
          format!("Unable to write to {}.", config_path.display())
        })?;
    }

    let config_str = fs::read_to_string(&config_path)
      .await
      .context("Unable to read config file.")?;

    let parsed_config = serde_yaml::from_str(&config_str)?;
    Ok(parsed_config)
  }

  fn create_from_sample() -> Self {
    let config_str = SAMPLE_CONFIG;
    serde_yaml::from_str(config_str).unwrap()
  }
}

fn to_length_value<'de, D>(
  deserializer: D,
) -> Result<LengthValue, D::Error>
where
  D: Deserializer<'de>,
{
  let str = String::deserialize(deserializer)?;
  LengthValue::from_str(&str).map_err(serde::de::Error::custom)
}

fn to_rect_delta<'de, D>(deserializer: D) -> Result<RectDelta, D::Error>
where
  D: Deserializer<'de>,
{
  let str = String::deserialize(deserializer)?;
  RectDelta::from_str(&str).map_err(serde::de::Error::custom)
}
