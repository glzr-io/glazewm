use std::{iter, path::PathBuf};

use clap::{
  error::{KindFormatter, RichFormatter},
  Args, Command, CommandFactory, Parser, Subcommand,
};
use serde::{Deserialize, Deserializer};
use tracing::Level;

use crate::common::{Direction, LengthValue, TilingDirection};

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
  Close,
  ChangeBorders(InvokeChangeBordersCommand),
  ChangeTilingDirection {
    #[clap(long = "tiling_dir")]
    tiling_direction: Option<TilingDirection>,

    #[clap(long)]
    toggle: bool,
  },
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
    toggle: bool,
  },
  SetFullscreen {
    #[clap(long)]
    toggle: bool,
  },
  SetMaximized {
    #[clap(long)]
    toggle: bool,
  },
  SetMinimized {
    #[clap(long)]
    toggle: bool,
  },
  SetTiling {
    #[clap(long)]
    toggle: bool,
  },
  ShellExec {
    #[clap(long, num_args = 1..)]
    command: Vec<String>,
  },
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
  WmChangeFocusMode {
    #[clap(long)]
    toggle: bool,
  },
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
pub struct InvokeChangeBordersCommand {
  #[clap(long)]
  width: Option<LengthValue>,

  #[clap(long)]
  height: Option<LengthValue>,
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
  #[clap(long)]
  width: Option<LengthValue>,

  #[clap(long)]
  height: Option<LengthValue>,
}
