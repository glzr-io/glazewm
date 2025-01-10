// Conditionally build the application with either a `windows` or `console`
// subsystem (default is `console`). This determines whether a console
// window is spawned on launch, if not already ran through a console.
#![cfg_attr(feature = "no_console", windows_subsystem = "windows")]
#![feature(iterator_try_collect)]
#![feature(once_cell_try)]

use std::{env, path::PathBuf};

use anyhow::{Context, Error, Result};
use tokio::{process::Command, signal};
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::{
  fmt::{self, writer::MakeWriterExt},
  layer::SubscriberExt,
};

use crate::{
  app_command::{AppCommand, InvokeCommand, Verbosity},
  common::platform::Platform,
  ipc_client::IpcClient,
  ipc_server::{ClientResponseData, IpcServer},
  sys_tray::SystemTray,
  user_config::UserConfig,
  wm::WindowManager,
  wm_event::WmEvent,
};

mod cleanup;
mod common;
mod containers;
mod ipc_server;
mod monitors;
mod sys_tray;
mod user_config;
mod windows;
mod wm;
mod wm_state;
mod workspaces;

/// Main entry point for the application.
///
/// Conditionally starts the WM or runs a CLI command based on the given
/// subcommand.
#[tokio::main]
async fn main() -> Result<()> {
  let args = std::env::args().collect::<Vec<_>>();
  let app_command = AppCommand::parse_with_default(&args);

  match app_command {
    AppCommand::Start {
      config_path,
      verbosity,
    } => {
      let res = start_wm(config_path, verbosity).await;

      // If unable to start the WM, the error is fatal and a message dialog
      // is shown.
      if let Err(err) = &res {
        error!("{:?}", err);
        Platform::show_error_dialog("Fatal error", &err.to_string());
      };

      res
    }
    _ => start_cli(args).await,
  }
}

async fn start_wm(
  config_path: Option<PathBuf>,
  verbosity: Verbosity,
) -> Result<()> {
  let error_log_dir = home::home_dir()
    .context("Unable to get home directory.")?
    .join(".glzr/glazewm/");

  let error_writer =
    tracing_appender::rolling::never(error_log_dir, "errors.log");

  let subscriber = tracing_subscriber::registry()
    .with(
      // Output to stdout with specified verbosity level.
      fmt::Layer::new()
        .with_writer(std::io::stdout.with_max_level(verbosity.level())),
    )
    .with(
      // Output to error log file.
      fmt::Layer::new()
        .with_writer(error_writer.with_max_level(Level::ERROR)),
    );

  tracing::subscriber::set_global_default(subscriber)?;

  info!(
    "Starting WM with log level {:?}.",
    verbosity.level().to_string()
  );

  // Ensure that only one instance of the WM is running.
  let _single_instance = Platform::new_single_instance()?;

  // Parse and validate user config.
  let mut config = UserConfig::new(config_path)?;

  // Start watcher process for restoring hidden windows on crash.
  start_watcher_process()?;

  // Add application icon to system tray.
  let mut tray = SystemTray::new(&config.path)?;

  let mut wm = WindowManager::new(&mut config)?;

  let mut ipc_server = IpcServer::start().await?;

  // Start listening for platform events after populating initial state.
  let mut event_listener = Platform::start_event_listener(&config)?;

  // Run startup commands.
  let startup_commands = config.value.general.startup_commands.clone();
  wm.process_commands(startup_commands, None, &mut config)?;

  loop {
    let res = tokio::select! {
      Some(_) = tray.exit_rx.recv() => {
        info!("Exiting through system tray.");
        break;
      },
      Some(_) = wm.exit_rx.recv() => {
        info!("Exiting through WM command.");
        break;
      },
      _ = signal::ctrl_c() => {
        info!("Received SIGINT signal.");
        break;
      },
      Some(event) = event_listener.event_rx.recv() => {
        debug!("Received platform event: {:?}", event);
        wm.process_event(event, &mut config)
      },
      Some((
        message,
        response_tx,
        disconnection_tx
      )) = ipc_server.message_rx.recv() => {
        info!("Received IPC message: {:?}", message);

        ipc_server.process_message(
          message,
          response_tx,
          disconnection_tx,
          &mut wm,
          &mut config,
        )
      },
      Some(wm_event) = wm.event_rx.recv() => {
        info!("Received WM event: {:?}", wm_event);

        // Update event listener when keyboard or mouse listener needs to
        // be changed.
        if matches!(
          wm_event,
          WmEvent::UserConfigChanged { .. }
            | WmEvent::BindingModesChanged { .. }
            | WmEvent::PauseChanged { .. }
        ) {
          event_listener.update(
            &config,
            &wm.state.binding_modes,
            wm.state.is_paused,
          );
        }

        ipc_server.process_event(wm_event)
      },
      Some(_) = tray.config_reload_rx.recv() => {
        wm.process_commands(
          vec![InvokeCommand::WmReloadConfig],
          None,
          &mut config,
        ).map(|_| ())
      },
    };

    if let Err(err) = res {
      error!("{:?}", err);
      Platform::show_error_dialog("Non-fatal error", &err.to_string());
    }
  }

  // Run shutdown commands.
  let shutdown_commands = config.value.general.shutdown_commands.clone();
  wm.process_commands(shutdown_commands, None, &mut config)?;

  wm.state.emit_event(WmEvent::ApplicationExiting);

  // Emit remaining WM events before exiting.
  while let Ok(wm_event) = wm.event_rx.try_recv() {
    info!("Emitting WM event before shutting down: {:?}", wm_event);

    if let Err(err) = ipc_server.process_event(wm_event) {
      warn!("{:?}", err);
    }
  }

  Ok(())
}

async fn start_cli(args: Vec<String>) -> Result<()> {
  let mut client = IpcClient::connect().await?;

  let message = args[1..].join(" ");
  client
    .send(&message)
    .await
    .context("Failed to send command to IPC server.")?;

  let client_response = client
    .client_response(&message)
    .await
    .context("Failed to receive response from IPC server.")?;

  match client_response.data {
    // For event subscriptions, omit the initial response message and
    // continuously output subsequent event messages.
    Some(ClientResponseData::EventSubscribe(data)) => loop {
      let event_subscription = client
        .event_subscription(&data.subscription_id)
        .await
        .context("Failed to receive response from IPC server.")?;

      println!("{}", serde_json::to_string(&event_subscription)?);
    },
    // For all other messages, output and exit when the first response
    // message is received.
    _ => {
      println!("{}", serde_json::to_string(&client_response)?);
    }
  }

  Ok(())
}

/// Launches watcher binary. This is a separate process that is responsible
/// for restoring hidden windows in case the main WM process crashes.
///
/// This assumes the watcher binary exists in the same directory as the WM
/// binary.
fn start_watcher_process() -> anyhow::Result<tokio::process::Child, Error>
{
  let watcher_path = env::current_exe()?
    .parent()
    .context("Failed to resolve path to the watcher process.")?
    .join("glazewm-watcher");

  Command::new(&watcher_path)
    .spawn()
    .context("Failed to start watcher process.")
}
