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
use tracing::{debug, error, info, warn, Level};
use tracing_subscriber::{
  fmt::{self, writer::MakeWriterExt},
  layer::SubscriberExt,
};
use wm_common::{AppCommand, InvokeCommand, Rect, Verbosity, WmEvent};
use wm_platform::{PlatformHook, WindowEvent};

// use wm_platform::Platform;
// use crate::{
//   ipc_server::IpcServer, sys_tray::SystemTray, user_config::UserConfig,
//   wm::WindowManager,
// };

// mod commands;
// mod events;
// mod ipc_server;
// mod models;
// mod pending_sync;
// mod sys_tray;
// mod traits;
// mod user_config;
// mod wm;
// mod wm_state;

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
    let (platform_hook, installer) = PlatformHook::new();

    let task_handle = rt.spawn(async {
      let res = start_wm(config_path, verbosity, platform_hook).await;

      // If unable to start the WM, the error is fatal and a message
      // dialog is shown.
      if let Err(err) = &res {
        error!("{:?}", err);
      }

      res
    });

    // Run event loop (blocks until shutdown). This must be on the main
    // thread for macOS compatibility.
    installer.run_dedicated_loop()?;

    // Wait for clean exit of the WM.
    rt.block_on(task_handle)?
  } else {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(wm_cli::start(args))
  }
}

async fn start_wm(
  config_path: Option<PathBuf>,
  verbosity: Verbosity,
  mut hook: PlatformHook,
) -> anyhow::Result<()> {
  setup_logging(&verbosity)?;

  // These will wait for the event loop to be ready
  // let mut mouse_listener = hook.create_mouse_listener().await?;
  // let mut keyboard_listener = hook.create_keyboard_listener().await?;
  let mut window_listener = hook.create_window_listener().await?;

  tracing::info!("Window manager started.");
  let monitors = hook.monitors()?;

  for monitor in monitors {
    tracing::info!("Monitor: {:?}", monitor.device_name());
    tracing::info!("Monitor: {:?}", monitor);
  }

  loop {
    // tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    // tracing::info!("Window manager running.");
    tokio::select! {
      _ = signal::ctrl_c() => {
        tracing::info!("Received SIGINT signal.");
        break;
      },
    //   Some(event) = mouse_listener.event_rx.recv() => {
    //     tracing::debug!("Received mouse event: {:?}", event);
    //     handle_event(PlatformEvent::Window(event)).await;
    //   },
    //   Some(event) = keyboard_listener.event_rx.recv() => {
    //     tracing::debug!("Received keyboard event: {:?}", event);
    //     handle_event(PlatformEvent::Keyboard(event)).await;
    //   },
      Some(event) = window_listener.event_rx.recv() => {
        tracing::info!("Received window event: {:?}", event);
        match event {
          WindowEvent::Show(window)=>{tracing::info!("Window shown: {:?}",window);}
          WindowEvent::Hide(window)=>{tracing::info!("Window hidden: {:?}",window);}
          WindowEvent::LocationChange(window)=>{
            tracing::info!("Window location changed: {:?}",window);
            // println!("Window title: {:?}", window.title());
            // println!("Window is visible: {:?}", window.is_visible());
            // println!("Window class name: {:?}", window.class_name());

            // Test resizing window to 400x300
            // println!("Attempting to resize window...");
            // match window.resize(Rect::from_xy(100, 100, 400, 300)) {
            //   Ok(()) => {
            //     println!("✅ Window resize successful!");
            //     tracing::info!("Successfully resized window to 400x300");
            //   }
            //   Err(e) => {
            //     println!("❌ Window resize failed: {}", e);
            //     tracing::error!("Failed to resize window: {}", e);
            //   }
            // }
          }
          WindowEvent::Minimize(window)=>{
            tracing::info!("Window minimized: {:?}",window);
            // println!("Window is visible: {:?}", window.is_visible());
          }
          WindowEvent::MinimizeEnd(window)=>{tracing::info!("Window deminimized: {:?}",window);}
          WindowEvent::TitleChange(window)=>{tracing::info!("Window title changed: {:?}",window);}
          WindowEvent::Focus(window)=>{
            tracing::info!("Window focused: {:?}",window);

            // Also test resizing on focus (different size)
            // println!("Window focused - testing resize to 600x400...");
            // match window.resize(Rect::from_xy(50, 50, 600, 400)) {
            //   Ok(()) => {
            //     println!("✅ Focus resize successful!");
            //     tracing::info!("Successfully resized focused window to 600x400");
            //   }
            //   Err(e) => {
            //     println!("❌ Focus resize failed: {}", e);
            //     tracing::error!("Failed to resize focused window: {}", e);
            //   }
            // }
          }
          WindowEvent::MoveOrResizeEnd(native_window) => todo!(),
          WindowEvent::MoveOrResizeStart(native_window) => todo!(),
        }
        // handle_event(PlatformEvent::Window(event)).await;
      },
    }
  }

  // Shutdown the platform hook.
  // hook.shutdown().await?;
  tracing::info!("Window manager shutting down.");

  Ok(())
}

// async fn start_wm(
//   config_path: Option<PathBuf>,
//   verbosity: Verbosity,
// ) -> anyhow::Result<()> {
//   setup_logging(&verbosity)?;

//   // Ensure that only one instance of the WM is running.
//   let _single_instance = Platform::new_single_instance()?;

//   // Parse and validate user config.
//   let mut config = UserConfig::new(config_path)?;

//   // Start watcher process for restoring hidden windows on crash.
//   start_watcher_process()?;

//   // Add application icon to system tray.
//   let mut tray = SystemTray::new(&config.path)?;

//   let mut wm = WindowManager::new(&mut config)?;

//   let mut ipc_server = IpcServer::start().await?;

//   // Start listening for platform events after populating initial state.
//   let mut event_listener =
// Platform::start_event_listener(&config.value)?;

//   // Run startup commands.
//   let startup_commands = config.value.general.startup_commands.clone();
//   wm.process_commands(&startup_commands, None, &mut config)?;

//   loop {
//     let res = tokio::select! {
//       Some(()) = tray.exit_rx.recv() => {
//         info!("Exiting through system tray.");
//         break;
//       },
//       Some(()) = wm.exit_rx.recv() => {
//         info!("Exiting through WM command.");
//         break;
//       },
//       _ = signal::ctrl_c() => {
//         info!("Received SIGINT signal.");
//         break;
//       },
//       Some(event) = event_listener.event_rx.recv() => {
//         debug!("Received platform event: {:?}", event);
//         wm.process_event(event, &mut config)
//       },
//       Some((
//         message,
//         response_tx,
//         disconnection_tx
//       )) = ipc_server.message_rx.recv() => {
//         info!("Received IPC message: {:?}", message);

//         if let Err(err) = ipc_server.process_message(
//           message,
//           &response_tx,
//           &disconnection_tx,
//           &mut wm,
//           &mut config,
//         ) {
//           error!("{:?}", err);
//         }

//         Ok(())
//       },
//       Some(wm_event) = wm.event_rx.recv() => {
//         debug!("Received WM event: {:?}", wm_event);

//         // Update event listener when keyboard or mouse listener needs
// to         // be changed.
//         if matches!(
//           wm_event,
//           WmEvent::UserConfigChanged { .. }
//             | WmEvent::BindingModesChanged { .. }
//             | WmEvent::PauseChanged { .. }
//         ) {
//           event_listener.update(
//             &config.value,
//             &wm.state.binding_modes,
//             wm.state.is_paused,
//           );
//         }

//         if let Err(err) = ipc_server.process_event(wm_event) {
//           error!("{:?}", err);
//         }

//         Ok(())
//       },
//       Some(()) = tray.config_reload_rx.recv() => {
//         wm.process_commands(
//           &vec![InvokeCommand::WmReloadConfig],
//           None,
//           &mut config,
//         ).map(|_| ())
//       },
//     };

//     if let Err(err) = res {
//       error!("{:?}", err);
//       Platform::show_error_dialog("Non-fatal error", &err.to_string());
//     }
//   }

//   run_cleanup(&mut wm, &mut config, &mut ipc_server)
// }

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

  info!(
    "Starting WM with log level {:?}.",
    verbosity.level().to_string()
  );

  Ok(())
}

// /// Launches watcher binary. This is a separate process that is
// responsible /// for restoring hidden windows in case the main WM process
// crashes. ///
// /// This assumes the watcher binary exists in the same directory as the
// WM /// binary.
// fn start_watcher_process() -> anyhow::Result<tokio::process::Child,
// Error> {
//   let watcher_path = env::current_exe()?
//     .parent()
//     .context("Failed to resolve path to the watcher process.")?
//     .join("glazewm-watcher");

//   Command::new(&watcher_path)
//     .spawn()
//     .context("Failed to start watcher process.")
// }

// /// Runs cleanup tasks when the WM is exiting.
// fn run_cleanup(
//   wm: &mut WindowManager,
//   config: &mut UserConfig,
//   ipc_server: &mut IpcServer,
// ) -> anyhow::Result<()> {
//   // Ensure that the WM is unpaused, otherwise, shutdown commands won't
// get   // executed.
//   wm.state.is_paused = false;

//   // Run shutdown commands.
//   let shutdown_commands =
// config.value.general.shutdown_commands.clone();   wm.process_commands(&
// shutdown_commands, None, config)?;

//   wm.state.emit_event(WmEvent::ApplicationExiting);

//   // Emit remaining WM events before exiting.
//   while let Ok(wm_event) = wm.event_rx.try_recv() {
//     info!("Emitting WM event before shutting down: {:?}", wm_event);

//     if let Err(err) = ipc_server.process_event(wm_event) {
//       warn!("{:?}", err);
//     }
//   }

//   Ok(())
// }
