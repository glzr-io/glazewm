// Conditionally build the application with either a `windows` or `console`
// subsystem (default is `console`). This determines whether a console
// window is spawned on launch, if not already ran through a console.
#![cfg_attr(feature = "windows_subsystem", windows_subsystem = "windows")]
#![feature(iterator_try_collect)]
#![feature(once_cell_try)]

use std::{env, path::PathBuf};

use anyhow::{Context, Result};
use app_command::Verbosity;
use ipc_client::IpcClient;
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
mod ipc_client;
mod ipc_server;
mod monitors;
mod user_config;
mod windows;
mod wm;
mod wm_event;
mod wm_state;
mod workspaces;

/// Main entry point for the application.
///
/// Conditionally runs the WM or CLI based on the given subcommand.
#[tokio::main]
async fn main() {
  let args = std::env::args().collect::<Vec<_>>();
  let app_command = AppCommand::parse_with_default(&args);

  let res = match app_command {
    AppCommand::Start {
      config_path,
      verbosity,
    } => start_wm(config_path, verbosity).await,
    AppCommand::Subscribe { .. } => start_cli(args, true).await,
    _ => start_cli(args, false).await,
  };

  if let Err(err) = res {
    error!("{}", err);
  }
}

async fn start_wm(
  config_path: Option<PathBuf>,
  verbosity: Verbosity,
) -> Result<()> {
  // Set log level based on verbosity flag.
  tracing_subscriber::fmt()
    .with_max_level(verbosity.level())
    .init();

  info!(
    "Starting WM with log level {:?}.",
    verbosity.level().to_string()
  );

  // Ensure that only one instance of the WM is running.
  let _ = Platform::new_single_instance()?;

  // Set the process-default DPI awareness.
  Platform::set_dpi_awareness()?;

  // Parse and validate user config.
  let mut config = UserConfig::read(config_path).await?;

  // Start watcher process for restoring hidden windows on crash.
  start_watcher_process()?;

  let mut wm = WindowManager::new(&config)?;

  // Start IPC server after populating initial WM state.
  let mut ipc_server = IpcServer::start().await?;

  // Start listening for platform events.
  let mut event_listener = Platform::start_event_listener(&config)?;

  loop {
    tokio::select! {
      Some(event) = event_listener.event_rx.recv() => {
        debug!("Received platform event: {:?}", event);
        let res = wm.process_event(event, &mut config);

        if let Err(err) = res {
          error!("Failed to process event: {:?}", err);
        }
      },
      Some((message, response_tx)) = ipc_server.message_rx.recv() => {
        info!("Received IPC message: {:?}", message);
        ipc_server.process_message(message, response_tx, &mut wm.state).await;
      },
      Some((command, subject_container_id)) = ipc_server.wm_command_rx.recv() => {
        info!("Received WM command via IPC: {:?}", command);
        let res = wm
          .process_commands(vec![command], subject_container_id, &mut config);

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

        ipc_server.process_event(wm_event, &mut wm.state).await;
      },
    }
  }
}

async fn start_cli(
  args: Vec<String>,
  is_subscribe_message: bool,
) -> Result<()> {
  tracing_subscriber::fmt().init();

  let mut client = IpcClient::connect().await?;

  client
    .send(args[1..].join(" "))
    .await
    .context("Failed to send command to IPC server.")?;

  let response = client
    .next_reply()
    .await
    .context("Failed to receive response from IPC server.")?;

  println!("{}", response);

  Ok(())
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
