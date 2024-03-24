use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::{
  fs,
  sync::{mpsc, Mutex},
};

use crate::{
  common::{LengthValue, RectDelta},
  monitors::Monitor,
  wm_command::WmCommand,
  workspaces::Workspace,
};

#[derive(Clone, Debug, Deserialize)]
pub struct KeybindingConfig {
  /// Keyboard shortcut to trigger the keybinding.
  pub bindings: Vec<String>,

  /// WM commands to run when the keybinding is triggered.
  pub commands: Vec<WmCommand>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BindingModeConfig {
  /// Name of the binding mode.
  pub name: String,

  /// Display name of the binding mode.
  pub display_name: Option<String>,

  /// Keybindings that will be active when the binding mode is active.
  pub keybindings: Vec<KeybindingConfig>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GapsConfig {
  /// Gap between adjacent windows.
  pub inner_gap: LengthValue,

  /// Gap between windows and the screen edge.
  pub outer_gap: RectDelta,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GeneralConfig {
  /// Whether to show floating windows as always on top.
  pub show_floating_on_top: bool,

  /// Center the cursor in the middle of a newly focused window.
  pub cursor_follows_focus: bool,

  /// Focus the window directly under the cursor at all times.
  pub focus_follows_cursor: bool,

  /// Amount by which to move floating windows
  pub floating_window_move_amount: LengthValue,

  /// If activated, by switching to the current workspace the previous
  /// focused workspace is activated.
  pub toggle_workspace_on_refocus: bool,

  /// Whether to enable window transition animations (on minimize, close).
  pub window_animations: WindowAnimations,

  /// Whether to center new floating windows
  pub center_new_floating_windows: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowAnimations {
  Enabled,
  Disabled,
  Unchanged,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FocusBorder {
  /// Whether to use a custom border color.
  pub enabled: bool,

  /// Border color of the window.
  pub color: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FocusBordersConfig {
  /// Border of the focused window.
  pub active: FocusBorder,

  /// Border of non-focused windows.
  pub inactive: FocusBorder,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WindowRuleConfig {
  pub match_process_name: Option<String>,
  pub match_class_name: Option<String>,
  pub match_title: Option<String>,
  pub commands: Vec<WmCommand>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WorkspaceConfig {
  pub name: String,
  pub display_name: Option<String>,
  pub bind_to_monitor: Option<String>,
  pub keep_alive: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ParsedConfig {
  pub binding_modes: Vec<BindingModeConfig>,
  pub focus_borders: FocusBordersConfig,
  pub gaps: GapsConfig,
  pub general: GeneralConfig,
  pub keybindings: Vec<KeybindingConfig>,
  pub window_rules: Vec<WindowRuleConfig>,
  pub workspaces: Vec<WorkspaceConfig>,
}

#[derive(Debug)]
pub struct UserConfig {
  pub value: ParsedConfig,
  pub changes_rx: mpsc::UnboundedReceiver<ParsedConfig>,
  pub changes_tx: mpsc::UnboundedSender<ParsedConfig>,
}

const SAMPLE_CONFIG: &str =
  include_str!("../../../resources/sample-config.yaml");

impl UserConfig {
  /// Read and validate the user config from the given path.
  pub async fn read(
    config_path: Option<String>,
  ) -> Result<Arc<Mutex<Self>>> {
    let default_config_path = home::home_dir()
      .context("Unable to get home directory.")?
      .join(".glzr/glazewm/config.yaml");

    let config_path = match config_path {
      Some(val) => PathBuf::from(val),
      None => default_config_path,
    };

    // Create new config file from sample if it doesn't exist.
    if !config_path.exists() {
      Self::create_sample(config_path.clone()).await?;
    }

    let config_str = fs::read_to_string(&config_path)
      .await
      .context("Unable to read config file.")?;

    let parsed_config = serde_yaml::from_str(&config_str)?;

    let (changes_tx, changes_rx) =
      mpsc::unbounded_channel::<ParsedConfig>();

    Ok(Arc::new(Mutex::new(Self {
      value: parsed_config,
      changes_rx,
      changes_tx,
    })))
  }

  /// Initialize a new config file from the sample config resource.
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

  // TODO: Maybe this should be on the impl of `UserConfig` instead.
  async fn refresh(&self) -> Result<()> {
    Ok(())
  }

  pub fn workspace_config_for_monitor(
    &self,
    monitor: &Monitor,
    active_workspaces: &Vec<Workspace>,
  ) -> Option<WorkspaceConfig> {
    let inactive_configs = self
      .value
      .workspaces
      .iter()
      .filter(|config| {
        active_workspaces
          .iter()
          .find(|workspace| workspace.config().name == config.name)
          .is_none()
      })
      .collect::<Vec<_>>();

    let bound_config = inactive_configs.iter().find(|&config| {
      config
        .bind_to_monitor
        .as_ref()
        .map(|monitor_name| monitor_name == &monitor.name())
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
      .cloned()
  }
}
