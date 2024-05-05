use std::{iter, path::PathBuf};

use anyhow::bail;
use clap::{error::KindFormatter, Args, Parser};
use serde::{Deserialize, Deserializer};
use tracing::Level;

use crate::{
  common::{Direction, LengthValue},
  containers::{traits::CommonGetters, Container},
  user_config::UserConfig,
  windows::{
    commands::update_window_state, traits::WindowGetters, WindowState,
  },
  wm_state::WmState,
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
#[clap(rename_all = "snake_case")]
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
#[clap(rename_all = "snake_case")]
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
    #[clap(long)]
    centered: bool,
  },
  SetFullscreen {
    #[clap(long)]
    maximized: bool,
  },
  SetMinimized,
  SetTiling,
  ShellExec {
    #[clap(long, num_args = 1..)]
    command: Vec<String>,
  },
  ToggleFloating {
    #[clap(long)]
    centered: bool,
  },
  ToggleFullscreen {
    #[clap(long)]
    maximized: bool,
  },
  ToggleMinimized,
  ToggleTiling,
  ToggleTilingDirection,
  WmDisableBindingMode {
    #[clap(long)]
    name: String,
  },
  WmExit,
  WmEnableBindingMode {
    #[clap(long)]
    name: String,
  },
  WmRedraw,
  WmReloadConfig,
  WmToggleFocusMode,
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
      InvokeCommand::Focus(_) => todo!(),
      InvokeCommand::Ignore => todo!(),
      InvokeCommand::Move(_) => todo!(),
      InvokeCommand::MoveWorkspace { direction } => todo!(),
      InvokeCommand::Resize(_) => todo!(),
      InvokeCommand::SetFloating { centered } => {
        match subject_container.as_window_container() {
          Ok(window) => update_window_state(
            window,
            WindowState::Floating,
            state,
            config,
          ),
          _ => Ok(()),
        }
      }
      InvokeCommand::SetFullscreen { maximized } => {
        match subject_container.as_window_container() {
          Ok(window) => match maximized {
            true => window.native().maximize(),
            // false => window.native().set_position(),
            false => Ok(()),
          },
          _ => Ok(()),
        }
      }
      InvokeCommand::SetMinimized => {
        match subject_container.as_window_container() {
          Ok(window) => window.native().minimize(),
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
      InvokeCommand::ShellExec { command } => todo!(),
      InvokeCommand::ToggleFloating { centered } => {
        match subject_container.as_window_container() {
          // TODO: Toggle floating.
          Ok(window) => update_window_state(
            window,
            WindowState::Floating,
            state,
            config,
          ),
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleFullscreen { maximized } => todo!(),
      InvokeCommand::ToggleMinimized => {
        match subject_container.as_window_container() {
          Ok(window) => {
            if window.native().is_minimized() {
              window.native().restore()
            } else {
              window.native().minimize()
            }
          }
          _ => Ok(()),
        }
      }
      InvokeCommand::ToggleTiling => todo!(),
      InvokeCommand::ToggleTilingDirection => todo!(),
      InvokeCommand::WmDisableBindingMode { name } => todo!(),
      InvokeCommand::WmExit => todo!(),
      InvokeCommand::WmEnableBindingMode { name } => todo!(),
      InvokeCommand::WmRedraw => {
        let root_container = state.root_container.clone();
        state.add_container_to_redraw(root_container.into());
        Ok(())
      }
      InvokeCommand::WmReloadConfig => todo!(),
      InvokeCommand::WmToggleFocusMode => todo!(),
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
