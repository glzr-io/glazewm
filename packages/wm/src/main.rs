// The `windows` or `console` subsystem (default is `console`) determines
// whether a console window is spawned on launch, if not already ran
// through a console. The following prevents this additional console window
// in release mode.
#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
#![warn(clippy::all, clippy::pedantic)]
#![feature(iterator_try_collect)]

use std::{env, path::PathBuf};

use anyhow::{Context, Error};
use tokio::{process::Command, signal};
use tracing::Level;
use tracing_subscriber::{
  fmt::{self, writer::MakeWriterExt},
  layer::SubscriberExt,
};
use wm_common::{AppCommand, Verbosity, WmEvent};
use wm_platform::{
  Dispatcher, EventLoop, KeybindingListener, MouseListener, PlatformEvent,
  WindowListener,
};

use crate::{
  ipc_server::IpcServer, user_config::UserConfig, wm::WindowManager,
};

mod commands;
mod events;
mod ipc_server;
mod models;
mod pending_sync;
// mod sys_tray;
mod traits;
mod user_config;
mod wm;
mod wm_state;

/// Main entry point for the application.
///
/// Conditionally starts the WM or runs a CLI command based on the given
/// subcommand.
fn main() -> anyhow::Result<()> {
  let args = std::env::args().collect::<Vec<_>>();
  let app_command = AppCommand::parse_with_default(&args);

  if let AppCommand::Start {
    config_path,
    verbosity,
  } = app_command
  {
    let rt = tokio::runtime::Runtime::new()?;
    let (event_loop, dispatcher) = EventLoop::new()?;

    let task_handle = std::thread::spawn(move || {
      rt.block_on(async {
        if let Err(err) =
          start_wm(config_path, verbosity, dispatcher).await
        {
          // If unable to start the WM, the error is fatal and a message
          // dialog is shown.
          // TODO: Show message dialog.
          tracing::error!("{:?}", err);
          return Err(err);
        }

        Ok(())
      })
    });

    // Run event loop (blocks until shutdown). This must be on the main
    // thread for macOS compatibility.
    event_loop.run()?;

    // Wait for clean exit of the WM.
    task_handle.join().unwrap()
  } else {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(wm_cli::start(args))
  }
}

async fn start_wm(
  config_path: Option<PathBuf>,
  verbosity: Verbosity,
  dispatcher: Dispatcher,
) -> anyhow::Result<()> {
  setup_logging(&verbosity)?;

  // Ensure that only one instance of the WM is running.
  // TODO: Implement this for macOS.
  // let _single_instance = SingleInstance::new()?;

  // Parse and validate user config.
  let mut config = UserConfig::new(config_path)?;

  // Add application icon to system tray.
  // TODO: Implement this for macOS.
  // let mut tray = SystemTray::new(&config.path)?;

  let mut wm = WindowManager::new(dispatcher.clone(), &mut config)?;

  let mut ipc_server = IpcServer::start().await?;

  // Start watcher process for restoring hidden windows on crash.
  start_watcher_process()?;

  // Start listening for platform events after populating initial state.
  let mut mouse_listener = MouseListener::new(dispatcher.clone())?;
  let mut window_listener = WindowListener::new(dispatcher.clone())?;
  let mut keybinding_listener = KeybindingListener::new(
    dispatcher.clone(),
    &config
      .active_keybinding_configs(&[], false)
      .flat_map(|kb| kb.bindings)
      .collect::<Vec<_>>(),
  )?;

  // Run startup commands.
  let startup_commands = config.value.general.startup_commands.clone();
  wm.process_commands(&startup_commands, None, &mut config)?;

  loop {
    let res = tokio::select! {
      _ = signal::ctrl_c() => {
        tracing::info!("Received SIGINT signal.");
        break;
      },
      Some(()) = wm.exit_rx.recv() => {
        tracing::info!("Exiting through WM command.");
        break;
      },
      // Some(()) = tray.exit_rx.recv() => {
      //   tracing::info!("Exiting through system tray.");
      //   break;
      // },
      Some(event) = window_listener.next_event() => {
        tracing::debug!("Received platform event: {:?}", event);
        wm.process_event(PlatformEvent::Window(event), &mut config)
      },
      Some(event) = mouse_listener.next_event() => {
        tracing::debug!("Received mouse event: {:?}", event);
        wm.process_event(PlatformEvent::MouseMove(event), &mut config)
      },
      Some(event) = keybinding_listener.next_event() => {
        tracing::debug!("Received keyboard event: {:?}", event);
        wm.process_event(PlatformEvent::Keybinding(event), &mut config)
      },
      Some((
        message,
        response_tx,
        disconnection_tx
      )) = ipc_server.message_rx.recv() => {
        tracing::info!("Received IPC message: {:?}", message);

        if let Err(err) = ipc_server.process_message(
          message,
          &response_tx,
          &disconnection_tx,
          &mut wm,
          &mut config,
        ) {
          tracing::error!("{:?}", err);
        }

        Ok(())
      },
      Some(wm_event) = wm.event_rx.recv() => {
        tracing::debug!("Received WM event: {:?}", wm_event);

        // Disable mouse and keybinding listeners when the WM is paused.
        if let WmEvent::PauseChanged { is_paused } = wm_event {
          mouse_listener.enable(!is_paused);
          keybinding_listener.enable(!is_paused);
        }

        // Update keybinding listener when keybindings change.
        if matches!(
          wm_event,
          WmEvent::UserConfigChanged { .. }
            | WmEvent::BindingModesChanged { .. }
        ) {
          keybinding_listener.update(
            &config
              .active_keybinding_configs(&wm.state.binding_modes, false)
              .flat_map(|kb| kb.bindings)
              .collect::<Vec<_>>(),
          );
        }

        if let Err(err) = ipc_server.process_event(wm_event) {
          tracing::error!("{:?}", err);
        }

        Ok(())
      },
      // Some(()) = tray.config_reload_rx.recv() => {
      //   wm.process_commands(
      //     &vec![InvokeCommand::WmReloadConfig],
      //     None,
      //     &mut config,
      //   ).map(|_| ())
      // },
    };

    if let Err(err) = res {
      tracing::error!("{:?}", err);
      // TODO: Show message dialog.
      // Platform::show_error_dialog("Non-fatal error", &err.to_string());
    }
  }

  tracing::info!("Window manager shutting down.");

  // Shutdown listeners first to ensure clean exit.
  // TODO: This should be handled automatically in the `stop_event_loop`
  // method.
  if let Err(err) = keybinding_listener.stop() {
    tracing::warn!("Failed to stop keybinding listener: {}", err);
  }

  run_cleanup(&mut wm, &mut config, &mut ipc_server);

  // Now shutdown the platform event loop.
  dispatcher.stop_event_loop()?;

  Ok(())
}

/// Initialize logging with the specified verbosity level.
///
/// Error logs are saved to `~/.glzr/glazewm/errors.log`.
fn setup_logging(verbosity: &Verbosity) -> anyhow::Result<()> {
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

  tracing::info!(
    "Starting WM with log level {:?}.",
    verbosity.level().to_string()
  );

  Ok(())
}

/// Launches watcher binary. This is a separate process that is responsible
/// for restoring hidden windows in case the main WM process crashes.
///
/// This assumes the watcher binary exists in the same directory as the
/// WM binary.
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

/// Runs cleanup tasks when the WM is exiting.
fn run_cleanup(
  wm: &mut WindowManager,
  config: &mut UserConfig,
  ipc_server: &mut IpcServer,
) -> anyhow::Result<()> {
  // Ensure that the WM is unpaused, otherwise, shutdown commands won't get
  // executed.
  wm.state.is_paused = false;

  // Run shutdown commands.
  let shutdown_commands = config.value.general.shutdown_commands.clone();
  wm.process_commands(&shutdown_commands, None, config)?;

  wm.state.emit_event(WmEvent::ApplicationExiting);

  // Emit remaining WM events before exiting.
  while let Ok(wm_event) = wm.event_rx.try_recv() {
    tracing::info!(
      "Emitting WM event before shutting down: {:?}",
      wm_event
    );

    if let Err(err) = ipc_server.process_event(wm_event) {
      tracing::warn!("{:?}", err);
    }
  }

  Ok(())
}
