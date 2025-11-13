use std::{collections::HashMap, env, fs, path::PathBuf};

use anyhow::{Context, Result};
use wm_common::{
  does_window_match, InvokeCommand, MatchType, ParsedConfig, WindowFilterType, WindowMatchConfig, WindowRuleConfig, WindowRuleEvent, WorkspaceConfig
};

use crate::{
  models::{Monitor, WindowContainer, Workspace},
  traits::{CommonGetters, WindowGetters},
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
  pub fn new(config_path: Option<PathBuf>) -> anyhow::Result<Self> {
    let default_config_path = home::home_dir()
      .context("Unable to get home directory.")?
      .join(".glzr/glazewm/config.yaml");

    let config_path = config_path
      .or_else(|| env::var("GLAZEWM_CONFIG_PATH").ok().map(PathBuf::from))
      .unwrap_or(default_config_path);

    let (config_value, config_str) = Self::read(&config_path)?;

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
  fn read(
    config_path: &PathBuf,
  ) -> anyhow::Result<(ParsedConfig, String)> {
    if !config_path.exists() {
      Self::create_sample(config_path)?;
    }

    let config_str = fs::read_to_string(config_path)
      .context("Unable to read config file.")?;

    // TODO: Improve error formatting of serde_yaml errors. Something
    // similar to https://github.com/AlexanderThaller/format_serde_error
    let config_value = serde_yaml::from_str(&config_str)?;

    Ok((config_value, config_str))
  }

  /// Initializes a new config file from the sample config resource.
  fn create_sample(config_path: &PathBuf) -> Result<()> {
    let parent_dir =
      config_path.parent().context("Invalid config path.")?;

    fs::create_dir_all(parent_dir).with_context(|| {
      format!("Unable to create directory {}.", &config_path.display())
    })?;

    fs::write(config_path, SAMPLE_CONFIG).with_context(|| {
      format!("Unable to write to {}.", config_path.display())
    })?;

    Ok(())
  }

  pub fn reload(&mut self) -> anyhow::Result<()> {
    let (config_value, config_str) = Self::read(&self.path)?;

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
        x_pos: None,
        y_pos: None,
        width: None,
        height: None,
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
          .or_insert_with(Vec::new)
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
      .get(event)
      .unwrap_or(&Vec::new())
      .iter()
      .filter(|rule| {
        // Skip if window has already ran the rule.
        if window.done_window_rules().contains(rule) {
          return false;
        }

        does_window_match(
          &rule.match_window,
          &WindowFilterType::Any,
          &window_title,
          &window_class,
          &window_process,
        )
      })
      .cloned()
      .collect::<Vec<_>>();

    Ok(pending_window_rules)
  }

  pub fn inactive_workspace_configs(
    &self,
    active_workspaces: &[Workspace],
  ) -> Vec<&WorkspaceConfig> {
    self
      .value
      .workspaces
      .iter()
      .filter(|config| {
        !active_workspaces
          .iter()
          .any(|workspace| workspace.config().name == config.name)
      })
      .collect()
  }

  pub fn workspace_config_for_monitor(
    &self,
    monitor: &Monitor,
    active_workspaces: &[Workspace],
  ) -> Option<&WorkspaceConfig> {
    let inactive_configs =
      self.inactive_workspace_configs(active_workspaces);

    inactive_configs.into_iter().find(|&config| {
      config
        .bind_to_monitor
        .as_ref()
        .is_some_and(|monitor_index| {
          monitor.index() == *monitor_index as usize
        })
    })
  }

  /// Gets the first inactive workspace config, prioritizing configs that
  /// don't have a monitor binding.
  pub fn next_inactive_workspace_config(
    &self,
    active_workspaces: &[Workspace],
  ) -> Option<&WorkspaceConfig> {
    let inactive_configs =
      self.inactive_workspace_configs(active_workspaces);

    inactive_configs
      .iter()
      .find(|config| config.bind_to_monitor.is_none())
      .or(inactive_configs.first())
      .copied()
  }

  pub fn workspace_config_index(
    &self,
    workspace_name: &str,
  ) -> Option<usize> {
    self
      .value
      .workspaces
      .iter()
      .position(|config| config.name == workspace_name)
  }

  pub fn sort_workspaces(&self, workspaces: &mut [Workspace]) {
    workspaces.sort_by_key(|workspace| {
      self.workspace_config_index(&workspace.config().name)
    });
  }

  pub fn outer_gaps_for_workspace(
    &self,
    workspace: &Workspace,
  ) -> &wm_common::RectDelta {
    let is_single_window = workspace.tiling_children().nth(1).is_none();

    if is_single_window {
      self
        .value
        .gaps
        .single_window_outer_gap
        .as_ref()
        .unwrap_or(&self.value.gaps.outer_gap)
    } else {
      &self.value.gaps.outer_gap
    }
  }
}
