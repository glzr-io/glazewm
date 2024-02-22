use clap::Parser;
use common::RectDelta;
use ipc_client;
use ipc_server::IpcServer;
use tokio::sync::mpsc;
use wineventhook::{EventFilter, WindowEventHook};
use wm_state::WmState;
use workspaces::Workspace;

use crate::cli::{Cli, CliCommand};

mod cli;
mod ipc_server;
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

async fn start_wm() {
  let (event_tx, mut event_rx) = mpsc::unbounded_channel();

  // TODO
  let (ipc_tx, mut ipc_rx) = mpsc::unbounded_channel::<i32>();

  let hook = WindowEventHook::hook(EventFilter::default(), event_tx)
    .await
    .unwrap();

  let ipc_server = IpcServer::new(ipc_tx);

  tokio::spawn(async move {
    let wm_state = WmState::new();

    // Wait and print events
    while let Some(event) = event_rx.recv().await {
      println!("{:#?}", event);
      wm_state.focus_mode();
    }
  });

  let mut foo = None;
  let mut bar = None;

  loop {
    tokio::select! {
        f = event_rx.recv() => foo = f,
        b = ipc_rx.recv() => bar = b,
    }
  }

  // Unhook the hook
  hook.unhook().await.unwrap();
}
