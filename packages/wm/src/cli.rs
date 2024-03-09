use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, arg_required_else_help = true)]
pub struct Cli {
  #[command(subcommand)]
  pub command: CliCommand,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
  /// Starts the window manager.
  Start {
    /// Custom path to user config file.
    #[clap(short, long)]
    config_path: Option<String>,
  },

  /// Queries the window manager's state.
  Query {
    windows: bool,
    monitors: bool,
    binding_mode: bool,
    focused_container: bool,
  },

  /// Runs a window manager command.
  Cmd {},
}
