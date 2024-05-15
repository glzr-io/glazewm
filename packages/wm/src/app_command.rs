use std::{iter, path::PathBuf};

use anyhow::bail;
use clap::{error::KindFormatter, Args, Parser};
use serde::{Deserialize, Deserializer};
use tracing::Level;

use crate::{
  common::{
    commands::shell_exec, Direction, LengthValue, ResizeDimension,
  },
  containers::{
    commands::toggle_tiling_direction, traits::CommonGetters, Container,
  },
  user_config::{FloatingStateConfig, FullscreenStateConfig, UserConfig},
  windows::{
    commands::{resize_window, toggle_window_state, update_window_state},
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

  /// Prints the window manager's state.
  ///
  /// Requires an already running instance of the window manager.
  Query {
    #[clap(subcommand)]
    command: QueryCommand,
  },

  /// Invokes a window manager command.
  ///
  /// Requires an already running instance of the window manager.
  Cmd {
    #[clap(subcommand)]
    command: InvokeCommand,
  },
}

impl AppCommand {
  /// Parses `AppCommand` from command line arguments.
  ///
  /// Defaults to `AppCommand::Start` if no arguments are provided.
  pub fn parse_with_default() -> Self {
    let args = std::env::args().skip(1);

    match args.len() == 0 {
      true => AppCommand::Start {
        config_path: None,
        verbosity: Verbosity {
          verbose: false,
          quiet: false,
        },
      },
      false => AppCommand::parse(),
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
  /// Prints all windows.
  Windows,
  /// Prints all monitors.
  Monitors,
  /// Prints the active binding modes.
  BindingMode,
  /// Prints the focused container (either a window or an empty workspace).
  Focused,
}

#[derive(Clone, Debug, Parser)]
pub enum InvokeCommand {
  AdjustBorders(InvokeAdjustBordersCommand),
  Close,
  Focus(InvokeFocusCommand),
  Ignore,
  Move(InvokeMoveCommand),
  MoveWorkspace {
    #[clap(long = "dir")]
    direction: Direction,
  },
  Resize(InvokeResizeCommand),
  SetFloating {
    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    show_on_top: Option<bool>,

    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    centered: Option<bool>,
  },
  SetFullscreen {
    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    show_on_top: Option<bool>,

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
    show_on_top: Option<bool>,

    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    centered: Option<bool>,
  },
  ToggleFullscreen {
    #[clap(long, default_missing_value = "true", require_equals = true, num_args = 0..=1)]
    show_on_top: Option<bool>,

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
          todo!()
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
      InvokeCommand::Move(_) => todo!(),
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
        show_on_top,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let floating_defaults =
            &config.value.window_state_defaults.floating;

          update_window_state(
            window,
            WindowState::Floating(FloatingStateConfig {
              centered: centered.unwrap_or(floating_defaults.centered),
              show_on_top: show_on_top
                .unwrap_or(floating_defaults.show_on_top),
            }),
            state,
            config,
          )
        }
        _ => Ok(()),
      },
      InvokeCommand::SetFullscreen {
        maximized,
        show_on_top,
        remove_title_bar,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let fullscreen_defaults =
            &config.value.window_state_defaults.fullscreen;

          update_window_state(
            window,
            WindowState::Fullscreen(FullscreenStateConfig {
              maximized: maximized
                .unwrap_or(fullscreen_defaults.maximized),
              show_on_top: show_on_top
                .unwrap_or(fullscreen_defaults.show_on_top),
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
            window,
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
        show_on_top,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let floating_defaults =
            &config.value.window_state_defaults.floating;

          toggle_window_state(
            window,
            WindowState::Floating(FloatingStateConfig {
              centered: centered.unwrap_or(floating_defaults.centered),
              show_on_top: show_on_top
                .unwrap_or(floating_defaults.show_on_top),
            }),
            state,
            config,
          )
        }
        _ => Ok(()),
      },
      InvokeCommand::ToggleFullscreen {
        maximized,
        show_on_top,
        remove_title_bar,
      } => match subject_container.as_window_container() {
        Ok(window) => {
          let fullscreen_defaults =
            &config.value.window_state_defaults.fullscreen;

          toggle_window_state(
            window,
            WindowState::Fullscreen(FullscreenStateConfig {
              maximized: maximized
                .unwrap_or(fullscreen_defaults.maximized),
              show_on_top: show_on_top
                .unwrap_or(fullscreen_defaults.show_on_top),
              remove_title_bar: remove_title_bar
                .unwrap_or(fullscreen_defaults.remove_title_bar),
            }),
            state,
            config,
          )
        }
        _ => Ok(()),
      },
      InvokeCommand::ToggleMinimized => {
        match subject_container.as_window_container() {
          Ok(window) => toggle_window_state(
            window,
            WindowState::Minimized,
            state,
            config,
          ),
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleTiling => {
        match subject_container.as_window_container() {
          Ok(window) => {
            toggle_window_state(window, WindowState::Tiling, state, config)
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleTilingDirection => {
        toggle_tiling_direction(subject_container, state, config)
      }
      InvokeCommand::WmDisableBindingMode { name } => todo!(),
      InvokeCommand::WmEnableBindingMode { name } => todo!(),
      InvokeCommand::WmExit => todo!(),
      InvokeCommand::WmRedraw => {
        let root_container = state.root_container.clone();
        state.add_container_to_redraw(root_container.into());
        Ok(())
      }
      InvokeCommand::WmReloadConfig => todo!(),
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

#[derive(Args, Clone, Debug)]
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

#[derive(Args, Clone, Debug)]
#[group(required = true, multiple = false)]
pub struct InvokeFocusCommand {
  #[clap(long = "dir")]
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

#[derive(Args, Clone, Debug)]
#[group(required = true, multiple = false)]
pub struct InvokeMoveCommand {
  /// Direction to move the window.
  #[clap(long = "dir")]
  direction: Option<Direction>,

  /// Name of workspace to move the window.
  #[clap(long)]
  workspace: Option<String>,
}

#[derive(Args, Clone, Debug)]
#[group(required = true, multiple = true)]
pub struct InvokeResizeCommand {
  #[clap(long, allow_hyphen_values = true)]
  width: Option<LengthValue>,

  #[clap(long, allow_hyphen_values = true)]
  height: Option<LengthValue>,
}
