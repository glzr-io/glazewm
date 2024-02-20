use clap::Parser;
use common::RectDelta;
use ipc_client;
use workspaces::Workspace;

use crate::cli::{Cli, CliCommand};

mod cli;
mod common;
mod containers;
mod monitors;
// mod user_config;
// mod windows;
mod wm_state;
mod workspaces;

#[tokio::main]
async fn main() {
  let cli = Cli::parse();

  match cli.command {
    CliCommand::Start { config } => {
      let workspace = Workspace::new(
        "name".into(),
        "display_name".into(),
        true,
        RectDelta::new(1, 2, 3, 4),
      );
    }
    _ => {
      let args = std::env::args_os();
      ipc_client::send_raw(args).unwrap()
    }
  }
}
