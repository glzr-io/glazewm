use std::path::PathBuf;

use serde::{Deserialize, Deserializer};
use tokio::fs;

use crate::common::{LengthValue, RectDelta};

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
pub struct BindingModeConfig {
  /// Name of the binding mode.
  pub name: String,

  /// Keybindings that will be active when the binding mode is active.
  pub keybindings: Vec<KeybindingConfig>,
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
  pub async fn read(config_path: Option<&str>) -> Self {
    let default_config_path = home::home_dir()
      .context("Unable to get home directory.")?
      .resolve(".glzr/glazewm/config.yaml");

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
      .context("Unable to read config file.");

    serde_yaml::from_str(config_str)
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
