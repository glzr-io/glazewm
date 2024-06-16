use std::{iter, path::PathBuf};

use anyhow::bail;
use clap::{error::KindFormatter, Args, Parser, ValueEnum};
use serde::{Deserialize, Deserializer, Serialize};
use tracing::Level;
use uuid::Uuid;

use crate::{
  common::{
    commands::{
      disable_binding_mode, enable_binding_mode, reload_config, shell_exec,
    },
    Direction, LengthValue, ResizeDimension,
  },
  containers::{
    commands::{focus_in_direction, toggle_tiling_direction},
    traits::CommonGetters,
    Container,
  },
  user_config::{FloatingStateConfig, FullscreenStateConfig, UserConfig},
  windows::{
    commands::{
      move_window_in_direction, move_window_to_workspace, resize_window,
      update_window_state,
    },
    traits::WindowGetters,
    WindowState,
  },
  wm_state::WmState,
  workspaces::commands::{focus_workspace, FocusWorkspaceTarget},
};

#[derive(Clone, Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub enum AppCommand {
  /// Starts the window manager.
  Start {
    /// Custom path to user config file.
    ///
    /// The default path is `%userprofile%/.glzr/glazewm/config.yaml`
    #[clap(short = 'c', long = "config", value_hint = clap::ValueHint::FilePath)]
    config_path: Option<PathBuf>,

    #[clap(flatten)]
    verbosity: Verbosity,
  },

  /// Retrieves and outputs a specific part of the window manager's state.
  ///
  /// Requires an already running instance of the window manager.
  #[clap(alias = "q")]
  Query {
    #[clap(subcommand)]
    command: QueryCommand,
  },

  /// Invokes a window manager command.
  ///
  /// Requires an already running instance of the window manager.
  #[clap(alias = "c")]
  Command {
    #[clap(short = 'c')]
    subject_container_id: Option<Uuid>,

    #[clap(subcommand)]
    command: InvokeCommand,
  },

  /// Subscribes to one or more WM events (e.g. `window_close`), and
  /// continuously outputs the incoming events.
  ///
  /// Requires an already running instance of the window manager.
  #[clap(alias = "su")]
  Subscribe {
    /// WM event(s) to subscribe to.
    #[clap(short = 'e', long, value_enum, num_args = 1..)]
    events: Vec<SubscribableEvent>,
  },

  /// Unsubscribes from a prior event subscription.
  ///
  /// Requires an already running instance of the window manager.
  #[clap(alias = "us")]
  Unsubscribe {
    /// Subscription ID to unsubscribe from.
    #[clap(long = "id")]
    subscription_id: Uuid,
  },
}

impl AppCommand {
  /// Parses `AppCommand` from command line arguments.
  ///
  /// Defaults to `AppCommand::Start` if no arguments are provided.
  pub fn parse_with_default(args: &Vec<String>) -> Self {
    match args.len() == 1 {
      true => AppCommand::Start {
        config_path: None,
        verbosity: Verbosity {
          verbose: false,
          quiet: false,
        },
      },
      false => AppCommand::parse_from(args),
    }
  }
}

/// Verbosity flags to be used with `#[command(flatten)]`.
#[derive(Args, Clone, Debug)]
#[clap(about = None, long_about = None)]
pub struct Verbosity {
  /// Enables verbose logging.
  #[clap(short = 'v', long, action)]
  verbose: bool,

  /// Disables logging.
  #[clap(short = 'q', long, action, conflicts_with = "verbose")]
  quiet: bool,
}

impl Verbosity {
  /// Gets the log level based on the verbosity flags.
  pub fn level(&self) -> Level {
    match (self.verbose, self.quiet) {
      (true, _) => Level::DEBUG,
      (_, true) => Level::ERROR,
      _ => Level::INFO,
    }
  }
}

#[derive(Clone, Debug, Parser)]
pub enum QueryCommand {
  /// Outputs metadata about the application (e.g. version number).
  AppMetadata,
  /// Outputs the active binding modes.
  BindingModes,
  /// Outputs the focused container (either a window or an empty workspace).
  Focused,
  /// Outputs all monitors.
  Monitors,
  /// Outputs all windows.
  Windows,
  /// Outputs all active workspaces.
  Workspaces,
}

#[derive(Clone, Debug, PartialEq, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum SubscribableEvent {
  All,
  ApplicationExiting,
  BindingModesChanged,
  FocusChanged,
  FocusedContainerMoved,
  MonitorAdded,
  MonitorRemoved,
  MonitorUpdated,
  TilingDirectionChanged,
  UserConfigChanged,
  WindowManaged,
  WindowUnmanaged,
  WorkspaceActivated,
  WorkspaceDeactivated,
  WorkspaceMoved,
}

#[derive(Clone, Debug, Parser, Serialize)]
pub enum InvokeCommand {
  AdjustBorders(InvokeAdjustBordersCommand),
  Close,
  Focus(InvokeFocusCommand),
  Ignore,
  Move(InvokeMoveCommand),
  MoveWorkspace {
    #[clap(long)]
    direction: Direction,
  },
  Resize(InvokeResizeCommand),
  SetFloating {
    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    shown_on_top: Option<bool>,

    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    centered: Option<bool>,
  },
  SetFullscreen {
    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    shown_on_top: Option<bool>,

    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    maximized: Option<bool>,

    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    remove_title_bar: Option<bool>,
  },
  SetMinimized,
  SetTiling,
  ShellExec {
    #[clap(long, num_args = 1.., allow_hyphen_values = true)]
    command: Vec<String>,
  },
  ToggleFloating {
    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    shown_on_top: Option<bool>,

    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    centered: Option<bool>,
  },
  ToggleFullscreen {
    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    shown_on_top: Option<bool>,

    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    maximized: Option<bool>,

    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    remove_title_bar: Option<bool>,
  },
  ToggleMinimized,
  ToggleTiling,
  ToggleTilingDirection,
  WmDisableBindingMode {
    #[clap(long)]
    name: String,
  },
  WmEnableBindingMode {
    #[clap(long)]
    name: String,
  },
  WmExit,
  WmRedraw,
  WmReloadConfig,
  WmToggleFocusState,
}

impl InvokeCommand {
  pub fn run(
    &self,
    subject_container: Container,
    state: &mut WmState,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    if subject_container.is_detached() {
      bail!("Cannot run command because subject container is detached.");
    }

    match self {
      InvokeCommand::AdjustBorders(_) => todo!(),
      InvokeCommand::Close => {
        match subject_container.as_window_container() {
          Ok(window) => window.native().close(),
          _ => Ok(()),
        }
      }
      InvokeCommand::Focus(args) => {
        if let Some(direction) = &args.direction {
          focus_in_direction(subject_container, direction, state)?;
        }

        if let Some(name) = &args.workspace {
          focus_workspace(
            FocusWorkspaceTarget::Name(name.to_string()),
            state,
            config,
          )?;
        }

        if args.next_workspace {
          focus_workspace(FocusWorkspaceTarget::Next, state, config)?;
        }

        if args.prev_workspace {
          focus_workspace(FocusWorkspaceTarget::Previous, state, config)?;
        }

        if args.recent_workspace {
          focus_workspace(FocusWorkspaceTarget::Recent, state, config)?;
        }

        Ok(())
      }
      InvokeCommand::Ignore => todo!(),
      InvokeCommand::Move(args) => {
        match subject_container.as_window_container() {
          Ok(window) => {
            if let Some(direction) = &args.direction {
              move_window_in_direction(
                window.clone(),
                direction,
                state,
                config,
              )?;
            };

            if let Some(workspace) = &args.workspace {
              move_window_to_workspace(window, workspace, state, config)?;
            }

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::MoveWorkspace { direction } => todo!(),
      InvokeCommand::Resize(args) => {
        match subject_container.as_window_container() {
          Ok(window) => {
            if let Some(width) = &args.width {
              resize_window(
                window.clone(),
                ResizeDimension::Width,
                width.clone(),
                state,
              )?
            }

            if let Some(height) = &args.height {
              resize_window(
                window,
                ResizeDimension::Height,
                height.clone(),
                state,
              )?
            }

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::SetFloating {
        centered,
        shown_on_top,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let floating_defaults =
            &config.value.window_behavior.state_defaults.floating;

          update_window_state(
            window.clone(),
            WindowState::Floating(FloatingStateConfig {
              centered: centered.unwrap_or(floating_defaults.centered),
              shown_on_top: shown_on_top
                .unwrap_or(floating_defaults.shown_on_top),
            }),
            state,
            config,
          )
        }
        _ => Ok(()),
      },
      InvokeCommand::SetFullscreen {
        maximized,
        shown_on_top,
        remove_title_bar,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let fullscreen_defaults =
            &config.value.window_behavior.state_defaults.fullscreen;

          update_window_state(
            window.clone(),
            WindowState::Fullscreen(FullscreenStateConfig {
              maximized: maximized
                .unwrap_or(fullscreen_defaults.maximized),
              shown_on_top: shown_on_top
                .unwrap_or(fullscreen_defaults.shown_on_top),
              remove_title_bar: remove_title_bar
                .unwrap_or(fullscreen_defaults.remove_title_bar),
            }),
            state,
            config,
          )
        }
        _ => Ok(()),
      },
      InvokeCommand::SetMinimized => {
        match subject_container.as_window_container() {
          Ok(window) => update_window_state(
            window.clone(),
            WindowState::Minimized,
            state,
            config,
          ),
          _ => Ok(()),
        }
      }
      InvokeCommand::SetTiling => {
        match subject_container.as_window_container() {
          Ok(window) => {
            update_window_state(window, WindowState::Tiling, state, config)
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::ShellExec { command } => {
        shell_exec(&command.join(" "))
      }
      InvokeCommand::ToggleFloating {
        centered,
        shown_on_top,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let floating_defaults =
            &config.value.window_behavior.state_defaults.floating;

          let target_state = WindowState::Floating(FloatingStateConfig {
            centered: centered.unwrap_or(floating_defaults.centered),
            shown_on_top: shown_on_top
              .unwrap_or(floating_defaults.shown_on_top),
          });

          update_window_state(
            window.clone(),
            window.toggled_state(target_state),
            state,
            config,
          )
        }
        _ => Ok(()),
      },
      InvokeCommand::ToggleFullscreen {
        maximized,
        shown_on_top,
        remove_title_bar,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let fullscreen_defaults =
            &config.value.window_behavior.state_defaults.fullscreen;

          let target_state =
            WindowState::Fullscreen(FullscreenStateConfig {
              maximized: maximized
                .unwrap_or(fullscreen_defaults.maximized),
              shown_on_top: shown_on_top
                .unwrap_or(fullscreen_defaults.shown_on_top),
              remove_title_bar: remove_title_bar
                .unwrap_or(fullscreen_defaults.remove_title_bar),
            });

          update_window_state(
            window.clone(),
            window.toggled_state(target_state),
            state,
            config,
          )
        }
        _ => Ok(()),
      },
      InvokeCommand::ToggleMinimized => {
        match subject_container.as_window_container() {
          Ok(window) => update_window_state(
            window.clone(),
            window.toggled_state(WindowState::Minimized),
            state,
            config,
          ),
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleTiling => {
        match subject_container.as_window_container() {
          Ok(window) => update_window_state(
            window.clone(),
            window.toggled_state(WindowState::Tiling),
            state,
            config,
          ),
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleTilingDirection => {
        toggle_tiling_direction(subject_container, state, config)
      }
      InvokeCommand::WmDisableBindingMode { name } => {
        disable_binding_mode(name, state);
        Ok(())
      }
      InvokeCommand::WmEnableBindingMode { name } => {
        enable_binding_mode(name, state, config)
      }
      InvokeCommand::WmExit => todo!(),
      InvokeCommand::WmRedraw => {
        let root_container = state.root_container.clone();
        state
          .pending_sync
          .containers_to_redraw
          .push(root_container.into());
        Ok(())
      }
      InvokeCommand::WmReloadConfig => reload_config(state, config),
      InvokeCommand::WmToggleFocusState => todo!(),
    }
  }
}

impl<'de> Deserialize<'de> for InvokeCommand {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    // Clap expects an array of string slices where the first argument is
    // the binary name/path. When deserializing commands from the user
    // config, we therefore have to prepend an additional empty argument.
    let unparsed = String::deserialize(deserializer)?;
    let unparsed_split = iter::once("").chain(unparsed.split_whitespace());

    InvokeCommand::try_parse_from(unparsed_split).map_err(|err| {
      // Format the error message and remove the "error: " prefix.
      let err_msg = err.apply::<KindFormatter>().to_string();
      serde::de::Error::custom(err_msg.trim_start_matches("error: "))
    })
  }
}

#[derive(Args, Clone, Debug, Serialize)]
#[group(required = true, multiple = true)]
pub struct InvokeAdjustBordersCommand {
  #[clap(long, allow_hyphen_values = true)]
  top: Option<LengthValue>,

  #[clap(long, allow_hyphen_values = true)]
  right: Option<LengthValue>,

  #[clap(long, allow_hyphen_values = true)]
  bottom: Option<LengthValue>,

  #[clap(long, allow_hyphen_values = true)]
  left: Option<LengthValue>,
}

#[derive(Args, Clone, Debug, Serialize)]
#[group(required = true, multiple = false)]
pub struct InvokeFocusCommand {
  #[clap(long)]
  direction: Option<Direction>,

  #[clap(long)]
  workspace: Option<String>,

  #[clap(long)]
  next_workspace: bool,

  #[clap(long)]
  prev_workspace: bool,

  #[clap(long)]
  recent_workspace: bool,
}

#[derive(Args, Clone, Debug, Serialize)]
#[group(required = true, multiple = false)]
pub struct InvokeMoveCommand {
  /// Direction to move the window.
  #[clap(long)]
  direction: Option<Direction>,

  /// Name of workspace to move the window.
  #[clap(long)]
  workspace: Option<String>,
}

#[derive(Args, Clone, Debug, Serialize)]
#[group(required = true, multiple = true)]
pub struct InvokeResizeCommand {
  #[clap(long, allow_hyphen_values = true)]
  width: Option<LengthValue>,

  #[clap(long, allow_hyphen_values = true)]
  height: Option<LengthValue>,
}
