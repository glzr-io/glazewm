use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
  app_command::InvokeCommand,
  common::{Color, LengthValue, RectDelta},
  containers::{traits::CommonGetters, WindowContainer},
  monitors::Monitor,
  windows::traits::WindowGetters,
  workspaces::Workspace,
};

/// Resource string for the sample config file.
const SAMPLE_CONFIG: &str =
  include_str!("../../../resources/assets/sample-config.yaml");

#[derive(Debug)]
pub struct UserConfig {
  /// Path to the user config file.
  pub path: PathBuf,

  /// Parsed user config value.
  pub value: ParsedConfig,

  /// Unparsed user config string.
  pub value_str: String,

  /// Hashmap of window rule event types (e.g. `WindowRuleEvent::Manage`)
  /// and the corresponding window rules of that type.
  window_rules_by_event: HashMap<WindowRuleEvent, Vec<WindowRuleConfig>>,
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

    let window_rules_by_event = Self::window_rules_by_event(&config_value);

    Ok(Self {
      path: config_path,
      value: config_value,
      value_str: config_str,
      window_rules_by_event,
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

    self.window_rules_by_event =
      Self::window_rules_by_event(&config_value);
    self.value = config_value;
    self.value_str = config_str;

    Ok(())
  }

  fn default_window_rules(
    config_value: &ParsedConfig,
  ) -> Vec<WindowRuleConfig> {
    let mut window_rules = Vec::new();

    let floating_defaults =
      &config_value.window_behavior.state_defaults.floating;

    // Default float rules.
    window_rules.push(WindowRuleConfig {
      commands: vec![InvokeCommand::SetFloating {
        centered: Some(floating_defaults.centered),
        shown_on_top: Some(floating_defaults.shown_on_top),
      }],
      match_window: vec![
        WindowMatchConfig {
          window_class: Some(MatchType::Equals { equals:
          // W10/W11 system dialog shown when moving and deleting files.
          "OperationStatusWindow".to_string(),
        }),
          ..WindowMatchConfig::default()
        },
        WindowMatchConfig {
          window_class: Some(MatchType::Equals { equals:
          // W10/W11 system dialogs (e.g. File Explorer save/open dialog).
          "#32770".to_string(),
        }),
          ..WindowMatchConfig::default()
        },
      ],
      on: vec![WindowRuleEvent::Manage],
      run_once: true,
    });

    // Default ignore rules.
    window_rules.push(WindowRuleConfig {
      commands: vec![InvokeCommand::Ignore],
      match_window: vec![
        WindowMatchConfig {
          window_process: Some(MatchType::Equals {
            equals: "SearchApp".to_string(),
          }),
          ..WindowMatchConfig::default()
        },
        WindowMatchConfig {
          window_process: Some(MatchType::Equals {
            equals: "SearchHost".to_string(),
          }),
          ..WindowMatchConfig::default()
        },
        WindowMatchConfig {
          window_process: Some(MatchType::Equals {
            equals: "ShellExperienceHost".to_string(),
          }),
          ..WindowMatchConfig::default()
        },
        WindowMatchConfig {
          window_process: Some(MatchType::Equals {
            // W10/11 start menu.
            equals: "StartMenuExperienceHost".to_string(),
          }),
          ..WindowMatchConfig::default()
        },
        WindowMatchConfig {
          window_process: Some(MatchType::Equals {
            // W10/11 screen snipping tool.
            equals: "ScreenClippingHost".to_string(),
          }),
          ..WindowMatchConfig::default()
        },
        WindowMatchConfig {
          window_process: Some(MatchType::Equals {
            // W11 lock screen.
            equals: "LockApp".to_string(),
          }),
          ..WindowMatchConfig::default()
        },
      ],
      on: vec![WindowRuleEvent::Manage],
      run_once: true,
    });

    window_rules
  }

  fn window_rules_by_event(
    config_value: &ParsedConfig,
  ) -> HashMap<WindowRuleEvent, Vec<WindowRuleConfig>> {
    let mut window_rules_by_event = HashMap::new();

    // Combine user-defined window rules with the default ones.
    let default_window_rules = Self::default_window_rules(config_value);
    let all_window_rules = config_value
      .window_rules
      .iter()
      .chain(default_window_rules.iter());

    for window_rule in all_window_rules {
      for event_type in &window_rule.on {
        window_rules_by_event
          .entry(event_type.clone())
          .or_insert_with(|| Vec::new())
          .push(window_rule.clone());
      }
    }

    window_rules_by_event
  }

  /// Window rules that should be applied to the window when the given
  /// event occurs.
  pub fn pending_window_rules(
    &self,
    window: &WindowContainer,
    event: &WindowRuleEvent,
  ) -> anyhow::Result<Vec<WindowRuleConfig>> {
    let window_title = window.native().title()?;
    let window_class = window.native().class_name()?;
    let window_process = window.native().process_name()?;

    let pending_window_rules = self
      .window_rules_by_event
      .get(&event)
      .unwrap_or(&Vec::new())
      .iter()
      .filter(|rule| {
        // Skip if window has already ran the rule.
        if window.done_window_rules().contains(&rule) {
          return false;
        }

        // Check if the window matches the rule.
        rule.match_window.iter().any(|match_config| {
          let is_process_match = match_config
            .window_process
            .as_ref()
            .map(|match_type| match_type.is_match(&window_process))
            .unwrap_or(true);

          let is_class_match = match_config
            .window_class
            .as_ref()
            .map(|match_type| match_type.is_match(&window_class))
            .unwrap_or(true);

          let is_title_match = match_config
            .window_title
            .as_ref()
            .map(|match_type| match_type.is_match(&window_title))
            .unwrap_or(true);

          is_process_match && is_class_match && is_title_match
        })
      })
      .cloned()
      .collect::<Vec<_>>();

    Ok(pending_window_rules)
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
        .map(|monitor_index| monitor.index() == *monitor_index as usize)
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

  /// Whether the floating window was created with a drag.
  #[serde(default = "default_bool::<false>")]
  pub is_tiling_drag: bool,
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
}

impl MatchType {
  /// Whether the given value is a match for the match type.
  fn is_match(&self, value: &str) -> bool {
    match self {
      MatchType::Equals { equals } => value == equals,
      MatchType::Includes { includes } => value.contains(includes),
      MatchType::Regex { regex } => regex::Regex::new(regex)
        .map(|re| re.is_match(value))
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
