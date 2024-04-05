use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use tracing::Level;

use crate::common::{Direction, LengthValue, TilingDirection};

#[derive(Parser, Debug)]
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
#[derive(Args, Debug)]
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

#[derive(Subcommand, Debug)]
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

#[derive(Subcommand, Debug)]
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
  SetFloating,
  SetFullscreen,
  SetMaximized,
  SetMinimized,
  SetTiling,
  SetTilingDirection {
    #[clap(long = "tiling_dir")]
    tiling_direction: TilingDirection,
  },
  ShellExec {
    #[clap(long, num_args = 1..)]
    command: Vec<String>,
  },
  ToggleFloating,
  ToggleFullscreen,
  ToggleMaximized,
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

#[derive(Args, Debug)]
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

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct InvokeMoveCommand {
  /// Direction to move the window.
  #[clap(long = "dir")]
  direction: Option<Direction>,

  /// Name of workspace to move the window.
  #[clap(long)]
  workspace: Option<String>,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = true)]
pub struct InvokeResizeCommand {
  #[clap(long)]
  width: Option<LengthValue>,

  #[clap(long)]
  height: Option<LengthValue>,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = true)]
pub struct InvokeAdjustBordersCommand {
  #[clap(long)]
  width: Option<LengthValue>,

  #[clap(long)]
  height: Option<LengthValue>,
}
