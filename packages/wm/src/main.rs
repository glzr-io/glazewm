use clap::Parser;
use tracing::info;

use crate::app_command::AppCommand;

mod app_command;
mod common;
// mod containers;
// mod ipc_server;
// mod monitors;
// mod user_config;
mod windows;
// mod wm;
// mod wm_command;
// mod wm_event;
// mod wm_state;
// mod workspaces;

#[tokio::main]
async fn main() {
  let app_command = AppCommand::parse_with_default();

  match app_command {
    AppCommand::Start {
      config_path,
      verbosity,
    } => {
      tracing_subscriber::fmt()
        .with_max_level(verbosity.level())
        .init();

      info!(
        "Starting WM with log level {:?}.",
        verbosity.level().to_string()
      );

      // start_wm(config_path).await?;
    }
    _ => todo!(),
  }

  // let config_path = None;

  // if let Err(err) = start_wm(config_path).await {
  //   eprintln!("Failed to start GlazeWM: {}", err);
  // }
}

async fn start_wm(config_path: Option<String>) -> Result<()> {
  // Ensure that only one instance of the WM is running.
  let _ = Platform::new_single_instance()?;

  // Parse and validate user config.
  let config = UserConfig::read(config_path).await?;

  let mut ipc_server = IpcServer::start().await?;

  // Start watcher process for restoring hidden windows on crash.
  start_watcher_process()?;

  let mut wm = WindowManager::start(&config).await?;

  // Start listening for platform events.
  let mut event_listener = Platform::new_event_listener(&config).await?;

  loop {
    let wm_state = wm.state.clone();
    let mut config = config.lock().await;

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
      Some(config) = config.changes_rx.recv() => {
        info!("Received user config update: {:?}", config);
        event_listener.update(&config);
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
