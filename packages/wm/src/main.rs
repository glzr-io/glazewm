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
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;
use user_config::ConfigReader;
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
  // TODO: Take log level and config path from `start` command arguments.
  tracing_subscriber::fmt()
    .with_env_filter(
      EnvFilter::from_env("LOG_LEVEL")
        .add_directive(LevelFilter::INFO.into()),
    )
    .init();

  let config_path = None;

  if let Err(err) = start_wm(config_path).await {
    eprintln!("Failed to start GlazeWM: {}", err);
  }
}

async fn start_wm(config_path: Option<String>) -> Result<()> {
  // Parse and validate user config.
  let mut config_reader = ConfigReader::read(config_path).await?;

  let mut ipc_server = IpcServer::start().await?;

  // Start watcher process for restoring hidden windows on crash.
  start_watcher_process()?;

  let mut wm = WindowManager::start(
    config_reader.config(),
    config_reader.changes_tx.clone(),
  )?;

  // Start listening for platform events.
  let mut event_listener =
    Platform::new_event_listener(config_reader.config()).await?;

  loop {
    let wm_state = wm.state.clone();

    tokio::select! {
      Some(event) = event_listener.event_rx.recv() => {
        info!("Received platform event: {:?}", event);
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
      Some(config) = config_reader.changes_rx.recv() => {
        info!("Received user config update: {:?}", config);
        event_listener.update();
      },
    }
  }
}

/// Launches watcher binary. This is a separate process that is responsible
/// for restoring hidden windows in case the main WM process crashes.
///
/// This assumes the watcher binary exists in the same directory as the WM
/// binary.
fn start_watcher_process() -> Result<Command> {
  let watcher_path = env::current_exe()?
    .parent()
    .context("Failed to resolve path to the watcher process.")?
    .join("watcher");

  Ok(Command::new(watcher_path))
}
