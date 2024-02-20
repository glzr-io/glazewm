use clap::Parser;
use ipc_client;

use crate::cli::{Cli, CliCommand};

mod cli;
mod common;
mod containers;
// mod monitors;
// mod user_config;
// mod windows;
mod wm_state;
mod workspaces;

#[tokio::main]
async fn main() {
  let cli = Cli::parse();

  match cli.command {
    CliCommand::Start { config } => {}
    _ => {
      let args = std::env::args_os();
      ipc_client::send_raw(args).unwrap()
    }
  }
}
