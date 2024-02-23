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
mod user_config;
// mod windows;
mod wm;
mod wm_state;
mod wm_command;
mod wm_event;
mod workspaces;

#[tokio::main]
async fn main() {
  let cli = Cli::parse();

  match cli.command {
    CliCommand::Start { config_path } => {
      let _ = start_wm(config_path).await;
    }
    _ => {
      let args = std::env::args_os();
      ipc_client::new().send_raw(args).unwrap()
    }
  }
}

async fn start_wm(config_path: Option<&str>) {
  let user_config = UserConfig::read(config_path).await;
  let event_listener = platform::EventListener::new().start().await;
  let ipc_server = IpcServer::new().start().await;
  let wm = WindowManager::new(user_config).start().await;

  loop {
    tokio::select! {
      Some(event) = event_listener.event_rx.recv() => {
        info!("Received platform event: {}", event);
        wm.process_event(event).await
      },
      Some(wm_event) = wm.event_rx.recv() => {
        info!("Received WM event: {}", wm_event);
        ipc_server.process_event(wm_event).await
      },
      Some(ipc_message) = ipc_server.message_rx.recv() => {
        info!("Received IPC message: {}", ipc_message);
        ipc_server.process_message(ipc_message, wm.state).await
      },
    }
  }
}
