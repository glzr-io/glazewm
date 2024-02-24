use std::env;

use anyhow::{Context, Result};
use clap::Parser;
use common::platform::EventListener;
use ipc_client::IpcClient;

use ipc_server::IpcServer;
use tokio::process::Command;
use tracing::info;
use user_config::UserConfig;
use wm::WindowManager;

use crate::cli::{Cli, CliCommand};

mod cli;
mod common;
mod containers;
mod ipc_server;
mod monitors;
mod user_config;
mod windows;
mod wm;
mod wm_command;
mod wm_event;
mod wm_state;
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
      IpcClient::new().send_raw(args).unwrap()
    }
  }
}

async fn start_wm(config_path: Option<&str>) {
  // Parse and validate user config.
  let user_config = UserConfig::read(config_path).await;

  let event_listener = EventListener::new().start().await;
  let ipc_server = IpcServer::new().start().await;

  // Start watcher process for restoring hidden windows on crash.
  start_watcher_process()?;

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

async fn start_watcher_process() -> Result<Command> {
  let watcher_path = env::var_os("CARGO_BIN_FILE_WATCHER")
    .context("Failed to resolve path to watcher process.")?;

  Command::new(watcher_path);
}
