use std::{iter, path::PathBuf};

use anyhow::{bail, Context};
use clap::{error::KindFormatter, Args, Parser, ValueEnum};
use serde::{Deserialize, Deserializer, Serialize};
use tracing::{warn, Level};
use uuid::Uuid;

use crate::{
  common::{
    commands::{
      cycle_focus, disable_binding_mode, enable_binding_mode,
      reload_config, shell_exec,
    },
    Direction, LengthValue, RectDelta, TilingDirection,
  },
  containers::{
    commands::{
      focus_in_direction, set_tiling_direction, toggle_tiling_direction,
    },
    traits::CommonGetters,
    Container,
  },
  monitors::commands::focus_monitor,
  user_config::{FloatingStateConfig, FullscreenStateConfig, UserConfig},
  windows::{
    commands::{
      ignore_window, move_window_in_direction, move_window_to_workspace,
      resize_window, set_window_position, set_window_position_to_center,
      set_window_size, update_window_state,
    },
    traits::WindowGetters,
    WindowState,
  },
  wm_state::WmState,
  workspaces::{
    commands::{focus_workspace, move_workspace_in_direction},
    WorkspaceTarget,
  },
};

const VERSION: &'static str = env!("VERSION_NUMBER");

#[derive(Clone, Debug, Parser)]
#[clap(author, version = VERSION, about, long_about = None)]
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
    #[clap(long = "id")]
    subject_container_id: Option<Uuid>,

    #[clap(subcommand)]
    command: InvokeCommand,
  },

  /// Subscribes to one or more WM events (e.g. `window_close`), and
  /// continuously outputs the incoming events.
  ///
  /// Requires an already running instance of the window manager.
  Sub {
    /// WM event(s) to subscribe to.
    #[clap(short = 'e', long, value_enum, num_args = 1..)]
    events: Vec<SubscribableEvent>,
  },

  /// Unsubscribes from a prior event subscription.
  ///
  /// Requires an already running instance of the window manager.
  Unsub {
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
  /// Outputs the focused container (either a window or an empty
  /// workspace).
  Focused,
  /// Outputs the tiling direction of the focused container.
  TilingDirection,
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
  MonitorUpdated,
  MonitorRemoved,
  TilingDirectionChanged,
  UserConfigChanged,
  WindowManaged,
  WindowUnmanaged,
  WorkspaceActivated,
  WorkspaceDeactivated,
  WorkspaceUpdated,
}

#[derive(Clone, Debug, Parser, PartialEq, Serialize)]
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
  Position(InvokePositionCommand),
  Resize(InvokeResizeCommand),
  SetFloating {
    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    shown_on_top: Option<bool>,

    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    centered: Option<bool>,

    #[clap(long, allow_hyphen_values = true)]
    x_pos: Option<i32>,

    #[clap(long, allow_hyphen_values = true)]
    y_pos: Option<i32>,

    #[clap(long, allow_hyphen_values = true)]
    width: Option<LengthValue>,

    #[clap(long, allow_hyphen_values = true)]
    height: Option<LengthValue>,
  },
  SetFullscreen {
    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    shown_on_top: Option<bool>,

    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    maximized: Option<bool>,
  },
  SetMinimized,
  SetTiling,
  SetTitleBarVisibility {
    #[clap(required = true, value_enum)]
    visibility: TitleBarVisibility,
  },
  ShellExec {
    #[clap(required = true, trailing_var_arg = true)]
    command: Vec<String>,
  },
  // Reuse `InvokeResizeCommand` struct.
  Size(InvokeResizeCommand),
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
  },
  ToggleMinimized,
  ToggleTiling,
  ToggleTilingDirection,
  SetTilingDirection {
    #[clap(required = true)]
    tiling_direction: TilingDirection,
  },
  WmCycleFocus {
    #[clap(long, default_value_t = false)]
    omit_fullscreen: bool,

    #[clap(long, default_value_t = true)]
    omit_minimized: bool,
  },
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
      InvokeCommand::AdjustBorders(args) => {
        match subject_container.as_window_container() {
          Ok(window) => {
            let args = args.clone();
            let border_delta = RectDelta::new(
              args.left.unwrap_or(LengthValue::from_px(0)),
              args.top.unwrap_or(LengthValue::from_px(0)),
              args.right.unwrap_or(LengthValue::from_px(0)),
              args.bottom.unwrap_or(LengthValue::from_px(0)),
            );

            window.set_border_delta(border_delta);
            state.pending_sync.containers_to_redraw.push(window.into());

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::Close => {
        match subject_container.as_window_container() {
          Ok(window) => {
            // Window handle might no longer be valid here.
            if let Err(err) = window.native().close() {
              warn!("Failed to close window: {:?}", err);
            }

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::Focus(args) => {
        if let Some(direction) = &args.direction {
          focus_in_direction(subject_container, direction, state)?;
        }

        if let Some(name) = &args.workspace {
          focus_workspace(
            WorkspaceTarget::Name(name.to_string()),
            state,
            config,
          )?;
        }

        if let Some(monitor_index) = &args.monitor {
          focus_monitor(*monitor_index, state, config)?;
        }

        if args.next_active_workspace {
          focus_workspace(WorkspaceTarget::NextActive, state, config)?;
        }

        if args.prev_active_workspace {
          focus_workspace(WorkspaceTarget::PreviousActive, state, config)?;
        }

        if args.next_workspace {
          focus_workspace(WorkspaceTarget::Next, state, config)?;
        }

        if args.prev_workspace {
          focus_workspace(WorkspaceTarget::Previous, state, config)?;
        }

        if args.recent_workspace {
          focus_workspace(WorkspaceTarget::Recent, state, config)?;
        }

        Ok(())
      }
      InvokeCommand::Ignore => {
        match subject_container.as_window_container() {
          Ok(window) => ignore_window(window, state),
          _ => Ok(()),
        }
      }
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

            if let Some(name) = &args.workspace {
              move_window_to_workspace(
                window.clone(),
                WorkspaceTarget::Name(name.to_string()),
                state,
                config,
              )?;
            }

            if args.next_active_workspace {
              move_window_to_workspace(
                window.clone(),
                WorkspaceTarget::NextActive,
                state,
                config,
              )?;
            }

            if args.prev_active_workspace {
              move_window_to_workspace(
                window.clone(),
                WorkspaceTarget::PreviousActive,
                state,
                config,
              )?;
            }

            if args.next_workspace {
              move_window_to_workspace(
                window.clone(),
                WorkspaceTarget::Next,
                state,
                config,
              )?;
            }

            if args.prev_workspace {
              move_window_to_workspace(
                window.clone(),
                WorkspaceTarget::Previous,
                state,
                config,
              )?;
            }

            if args.recent_workspace {
              move_window_to_workspace(
                window,
                WorkspaceTarget::Recent,
                state,
                config,
              )?;
            }

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::MoveWorkspace { direction } => {
        let workspace =
          subject_container.workspace().context("No workspace.")?;

        move_workspace_in_direction(
          workspace,
          direction.clone(),
          state,
          config,
        )
      }
      InvokeCommand::Position(args) => {
        match subject_container.as_window_container() {
          Ok(window) => match args.centered {
            true => set_window_position_to_center(window, state),
            false => set_window_position(
              window,
              args.x_pos.clone(),
              args.y_pos.clone(),
              state,
            ),
          },
          _ => Ok(()),
        }
      }
      InvokeCommand::Resize(args) => {
        match subject_container.as_window_container() {
          Ok(window) => resize_window(
            window,
            args.width.clone(),
            args.height.clone(),
            state,
          ),
          _ => Ok(()),
        }
      }
      InvokeCommand::SetFloating {
        centered,
        shown_on_top,
        x_pos,
        y_pos,
        width,
        height,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let floating_defaults =
            &config.value.window_behavior.state_defaults.floating;
          let is_centered = centered.unwrap_or(floating_defaults.centered);

          let window = update_window_state(
            window.clone(),
            WindowState::Floating(FloatingStateConfig {
              centered: is_centered,
              shown_on_top: shown_on_top
                .unwrap_or(floating_defaults.shown_on_top),
            }),
            state,
            config,
          )?;

          if width.is_some() || height.is_some() {
            set_window_size(
              window.clone(),
              width.clone(),
              height.clone(),
              state,
            )?;
          }

          if is_centered {
            set_window_position_to_center(window, state)?;
          } else if x_pos.is_some() || y_pos.is_some() {
            set_window_position(
              window.clone(),
              x_pos.clone(),
              y_pos.clone(),
              state,
            )?;
          }

          Ok(())
        }
        _ => Ok(()),
      },
      InvokeCommand::SetFullscreen {
        maximized,
        shown_on_top,
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
            }),
            state,
            config,
          )?;

          Ok(())
        }
        _ => Ok(()),
      },
      InvokeCommand::SetMinimized => {
        match subject_container.as_window_container() {
          Ok(window) => {
            update_window_state(
              window.clone(),
              WindowState::Minimized,
              state,
              config,
            )?;

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::SetTiling => {
        match subject_container.as_window_container() {
          Ok(window) => {
            update_window_state(
              window,
              WindowState::Tiling,
              state,
              config,
            )?;

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::SetTitleBarVisibility { visibility } => {
        match subject_container.as_window_container() {
          Ok(window) => {
            _ = window.native().set_title_bar_visibility(
              if *visibility == TitleBarVisibility::Shown {
                true
              } else {
                false
              },
            );
            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::ShellExec { command } => {
        shell_exec(&command.join(" "))
      }
      InvokeCommand::Size(args) => {
        match subject_container.as_window_container() {
          Ok(window) => set_window_size(
            window,
            args.width.clone(),
            args.height.clone(),
            state,
          ),
          _ => Ok(()),
        }
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
            window.toggled_state(target_state, config),
            state,
            config,
          )?;

          Ok(())
        }
        _ => Ok(()),
      },
      InvokeCommand::ToggleFullscreen {
        maximized,
        shown_on_top,
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
            });

          update_window_state(
            window.clone(),
            window.toggled_state(target_state, config),
            state,
            config,
          )?;

          Ok(())
        }
        _ => Ok(()),
      },
      InvokeCommand::ToggleMinimized => {
        match subject_container.as_window_container() {
          Ok(window) => {
            update_window_state(
              window.clone(),
              window.toggled_state(WindowState::Minimized, config),
              state,
              config,
            )?;

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleTiling => {
        match subject_container.as_window_container() {
          Ok(window) => {
            update_window_state(
              window.clone(),
              window.toggled_state(WindowState::Tiling, config),
              state,
              config,
            )?;

            Ok(())
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleTilingDirection => {
        toggle_tiling_direction(subject_container, state, config)
      }
      InvokeCommand::SetTilingDirection { tiling_direction } => {
        set_tiling_direction(
          subject_container,
          state,
          config,
          tiling_direction.clone(),
        )
      }
      InvokeCommand::WmCycleFocus {
        omit_fullscreen,
        omit_minimized,
      } => cycle_focus(*omit_fullscreen, *omit_minimized, state, config),
      InvokeCommand::WmDisableBindingMode { name } => {
        disable_binding_mode(name, state);
        Ok(())
      }
      InvokeCommand::WmEnableBindingMode { name } => {
        enable_binding_mode(name, state, config)
      }
      InvokeCommand::WmExit => {
        state.emit_exit();
        Ok(())
      }
      InvokeCommand::WmRedraw => {
        let root_container = state.root_container.clone();
        state
          .pending_sync
          .containers_to_redraw
          .push(root_container.into());

        Ok(())
      }
      InvokeCommand::WmReloadConfig => reload_config(state, config),
    }
  }

  pub fn run_multiple(
    commands: Vec<InvokeCommand>,
    subject_container: Container,
    state: &mut WmState,
    config: &mut UserConfig,
  ) -> anyhow::Result<Uuid> {
    let mut current_subject_container = subject_container;

    for command in commands {
      command.run(current_subject_container.clone(), state, config)?;

      // Update the subject container in case the container type changes.
      // For example, when going from a tiling to a floating window.
      current_subject_container =
        match current_subject_container.is_detached() {
          false => current_subject_container,
          true => {
            match state.container_by_id(current_subject_container.id()) {
              Some(container) => container,
              None => break,
            }
          }
        }
    }

    Ok(current_subject_container.id())
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

#[derive(Clone, Debug, PartialEq, Serialize, ValueEnum)]
#[clap(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TitleBarVisibility {
  Shown,
  Hidden,
}

#[derive(Args, Clone, Debug, PartialEq, Serialize)]
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

#[derive(Args, Clone, Debug, PartialEq, Serialize)]
#[group(required = true, multiple = false)]
pub struct InvokeFocusCommand {
  #[clap(long)]
  direction: Option<Direction>,

  #[clap(long)]
  workspace: Option<String>,

  #[clap(long)]
  monitor: Option<usize>,

  #[clap(long)]
  next_active_workspace: bool,

  #[clap(long)]
  prev_active_workspace: bool,

  #[clap(long)]
  next_workspace: bool,

  #[clap(long)]
  prev_workspace: bool,

  #[clap(long)]
  recent_workspace: bool,
}

#[derive(Args, Clone, Debug, PartialEq, Serialize)]
#[group(required = true, multiple = false)]
pub struct InvokeMoveCommand {
  /// Direction to move the window.
  #[clap(long)]
  direction: Option<Direction>,

  /// Name of workspace to move the window.
  #[clap(long)]
  workspace: Option<String>,

  #[clap(long)]
  next_active_workspace: bool,

  #[clap(long)]
  prev_active_workspace: bool,

  #[clap(long)]
  next_workspace: bool,

  #[clap(long)]
  prev_workspace: bool,

  #[clap(long)]
  recent_workspace: bool,
}

#[derive(Args, Clone, Debug, PartialEq, Serialize)]
#[group(required = true, multiple = true)]
pub struct InvokeResizeCommand {
  #[clap(long, allow_hyphen_values = true)]
  width: Option<LengthValue>,

  #[clap(long, allow_hyphen_values = true)]
  height: Option<LengthValue>,
}

#[derive(Args, Clone, Debug, PartialEq, Serialize)]
#[group(required = true, multiple = true)]
pub struct InvokePositionCommand {
  #[clap(long, action)]
  centered: bool,

  #[clap(long, allow_hyphen_values = true)]
  x_pos: Option<i32>,

  #[clap(long, allow_hyphen_values = true)]
  y_pos: Option<i32>,
}
