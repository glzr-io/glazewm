use std::{env, sync::Arc};

use anyhow::{Context, Result};
use clap::Parser;
use common::platform::Platform;
use ipc_client::IpcClient;

use ipc_server::IpcServer;
use tokio::{
  process::Command,
  sync::{mpsc, Mutex},
};
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
      if let Err(err) = start_wm(config_path).await {
        eprintln!("Error starting WM: {}", err);
      }
    }
    _ => {
      let args = std::env::args_os();
      IpcClient::connect()
        .await
        .unwrap()
        .send_raw(args)
        .await
        .unwrap()
    }
  }
}

async fn start_wm(config_path: Option<String>) -> Result<()> {
  // Parse and validate user config.
  let config = Arc::new(Mutex::new(UserConfig::read(config_path).await?));
  let (config_changes_tx, config_changes_rx) =
    mpsc::unbounded_channel::<UserConfig>();

  // Start listening for platform events.
  let mut event_listener =
    Platform::new_event_listener(config.clone(), config_changes_rx)
      .await?;

  let mut ipc_server = IpcServer::start().await?;

  // Start watcher process for restoring hidden windows on crash.
  start_watcher_process()?;

  let mut wm = WindowManager::start(config, config_changes_tx).await?;

  loop {
    let wm_state = wm.state.clone();

    tokio::select! {
      Some(event) = event_listener.event_rx.recv() => {
        info!("Received platform event: {:?}", event);
        wm_state.lock().await.add_monitor();
        wm_state.lock().await.add_monitor();
        wm_state.lock().await.add_monitor();
        wm.process_event(event).await
      },
      Some(wm_event) = wm.event_rx.recv() => {
        info!("Received WM event: {:?}", wm_event);
        ipc_server.process_event(wm_event).await
      },
      Some(ipc_message) = ipc_server.message_rx.recv() => {
        info!("Received IPC message: {:?}", ipc_message);
        ipc_server.process_message(ipc_message, wm_state).await
      },
      Some(wm_command) = ipc_server.wm_command_rx.recv() => {
        info!("Received WM command via IPC: {:?}", wm_command);
        wm.process_command(wm_command).await
      },
    }
  }
}

fn start_watcher_process() -> Result<Command> {
  let watcher_path = env::var_os("CARGO_BIN_FILE_WATCHER")
    .context("Failed to resolve path to the watcher process.")?;

  Ok(Command::new(watcher_path))
}
