#![feature(iterator_try_collect)]

use std::{env, path::PathBuf};

use anyhow::{Context, Result};
use tokio::process::Command;
use tracing::{debug, error, info};

use common::platform::Platform;
use ipc_server::IpcServer;
use user_config::UserConfig;
use wm::WindowManager;

use crate::{app_command::AppCommand, wm_event::WmEvent};

mod app_command;
mod common;
mod containers;
mod ipc_server;
mod monitors;
mod user_config;
mod windows;
mod wm;
mod wm_event;
mod wm_state;
mod workspaces;

#[tokio::main]
async fn main() {
  match AppCommand::parse_with_default() {
    AppCommand::Start {
      config_path,
      verbosity,
    } => {
      // Set log level based on verbosity flag.
      tracing_subscriber::fmt()
        .with_max_level(verbosity.level())
        .init();

      info!(
        "Starting WM with log level {:?}.",
        verbosity.level().to_string()
      );

      if let Err(err) = start_wm(config_path).await {
        error!("Failed to start GlazeWM: {}", err);
      }
    }
    _ => todo!(),
  }
}

async fn start_wm(config_path: Option<PathBuf>) -> Result<()> {
  // Ensure that only one instance of the WM is running.
  let _ = Platform::new_single_instance()?;

  // Set the process-default DPI awareness.
  Platform::set_dpi_awareness()?;

  // Parse and validate user config.
  let config = UserConfig::read(config_path).await?;

  let mut ipc_server = IpcServer::start().await?;

  // Start watcher process for restoring hidden windows on crash.
  start_watcher_process()?;

  let mut wm = WindowManager::start(&config).await?;

  // Start listening for platform events.
  let mut event_listener = Platform::new_event_listener(&config).await?;

  loop {
    let mut config = config.lock().await;

    tokio::select! {
      Some(event) = event_listener.event_rx.recv() => {
        debug!("Received platform event: {:?}", event);
        let res = wm.process_event(event, &mut config).await;

        if let Err(err) = res {
          error!("Failed to process event: {:?}", err);
        }
      },
      Some(ipc_message) = ipc_server.message_rx.recv() => {
        info!("Received IPC message: {:?}", ipc_message);
        ipc_server.process_message(ipc_message, wm.state.clone()).await;
      },
      Some(wm_command) = ipc_server.wm_command_rx.recv() => {
        info!("Received WM command via IPC: {:?}", wm_command);
        let (command, subject_container_id) = wm_command;
        let res = wm
          .process_commands(vec![command], subject_container_id, &mut config)
          .await;

        if let Err(err) = res {
          error!("Failed to process command: {:?}", err);
        }
      },
      Some(_) = config.changes_rx.recv() => {
        info!("Received user config update: {:?}", config);
        event_listener.update(&config, &Vec::new());
      },
      Some(wm_event) = wm.event_rx.recv() => {
        info!("Received WM event: {:?}", wm_event);

        if let WmEvent::BindingModesChanged { active_binding_modes } = &wm_event {
          event_listener.update(&config, active_binding_modes);
        }

        ipc_server.process_event(wm_event).await;
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
