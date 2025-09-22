use std::{
  cell::RefCell,
  fmt::{self, Display},
  path::Path,
  str::FromStr,
  time::Duration,
};

use anyhow::Context;
use tokio::sync::mpsc;
use tracing::{info, warn};
use tray_icon::{
  menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
};
use wm_platform::Dispatcher;

#[derive(Debug, Clone, Eq, PartialEq)]
enum TrayMenuEvent {
  ReloadConfig,
  ShowConfigFolder,
  Exit,
}

impl Display for TrayMenuEvent {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      TrayMenuEvent::ReloadConfig => write!(f, "reload_config"),
      TrayMenuEvent::ShowConfigFolder => write!(f, "show_config_folder"),
      TrayMenuEvent::Exit => write!(f, "exit"),
    }
  }
}

impl FromStr for TrayMenuEvent {
  type Err = anyhow::Error;

  fn from_str(event: &str) -> Result<Self, Self::Err> {
    let parts: Vec<&str> = event.split('_').collect();

    match parts.as_slice() {
      ["show", "config", "folder"] => Ok(Self::ShowConfigFolder),
      ["reload", "config"] => Ok(Self::ReloadConfig),
      ["exit"] => Ok(Self::Exit),
      _ => anyhow::bail!("Invalid tray menu event: {}", event),
    }
  }
}
pub struct SystemTray {
  pub config_reload_rx: mpsc::UnboundedReceiver<()>,
  pub exit_rx: mpsc::UnboundedReceiver<()>,
  icon_thread: Option<std::thread::JoinHandle<anyhow::Result<()>>>,
  tray_icon: TrayIcon,
}

impl SystemTray {
  /// Install the system tray on the main thread after the run loop starts.
  ///
  /// This schedules tray creation onto the dispatcher and stores it in a
  /// global so it stays alive for the app lifetime.
  pub fn install(dispatcher: &Dispatcher) -> anyhow::Result<()> {
    thread_local! {
      static TRAY_INSTANCE: RefCell<Option<SystemTray>> = RefCell::new(None);
    }

    let dispatcher_clone = dispatcher.clone();
    dispatcher.dispatch(move || match Self::new(&dispatcher_clone) {
      Ok(tray) => {
        TRAY_INSTANCE.with(|cell| {
          *cell.borrow_mut() = Some(tray);
        });
        println!("System tray installed.");
      }
      Err(err) => {
        println!("Failed to create system tray: {:?}", err);
      }
    })?;

    Ok(())
  }

  pub fn new(
    // config_path: &Path,
    _dispatcher: &Dispatcher,
  ) -> anyhow::Result<Self> {
    let (_exit_tx, exit_rx) = mpsc::unbounded_channel();
    let (_config_reload_tx, config_reload_rx) = mpsc::unbounded_channel();
    // let config_dir = config_path
    //   .parent()
    //   .context("Invalid config path.")?
    //   .to_owned();

    let reload_config_item = MenuItem::with_id(
      TrayMenuEvent::ReloadConfig,
      "Reload config",
      true,
      None,
    );

    let config_dir_item = MenuItem::with_id(
      TrayMenuEvent::ShowConfigFolder,
      "Show config folder",
      true,
      None,
    );

    let exit_item =
      MenuItem::with_id(TrayMenuEvent::Exit, "Exit", true, None);

    let tray_menu = Menu::new();
    tray_menu
      .append_items(&[
        &reload_config_item,
        &config_dir_item,
        &PredefinedMenuItem::separator(),
        &exit_item,
      ])
      .unwrap();

    let path = concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/../../resources/assets/icon.png"
    );

    let icon = Self::load_icon(Path::new(path)).unwrap();

    let tray_icon = TrayIconBuilder::new()
      .with_menu(Box::new(tray_menu))
      .with_tooltip(format!("GlazeWM v{}", env!("VERSION_NUMBER")))
      .with_icon(icon)
      .build()
      .unwrap();

    tray_icon.set_show_menu_on_left_click(true);

    TrayIconEvent::set_event_handler(Some(move |event| {
      println!("Tray icon event: {:?}", event);
    }));

    MenuEvent::set_event_handler(Some(move |event| {
      println!("Tray menu event: {:?}", event);
    }));

    // Also listen on channels as a fallback in case handler was already
    // set elsewhere.
    let mut icon_thread_handle = None;
    {
      let tray_rx = TrayIconEvent::receiver().clone();
      let menu_rx = MenuEvent::receiver().clone();
      icon_thread_handle = Some(std::thread::spawn(move || loop {
        if let Ok(event) = tray_rx.try_recv() {
          println!("Tray icon event (rx): {:?}", event);
        }
        if let Ok(event) = menu_rx.try_recv() {
          println!("Tray menu event (rx): {:?}", event);
        }
        std::thread::sleep(Duration::from_millis(50));
      }));
    }

    // Spawn thread to handle menu events and forward them to channels
    // let icon_thread = std::thread::spawn(move || {
    //   let menu_event_rx = MenuEvent::receiver();

    //   loop {
    //     if let Ok(event) = menu_event_rx.try_recv() {
    //       let event_res = match
    // TrayMenuEvent::from_str(event.id.as_ref())       {
    //         Ok(TrayMenuEvent::ShowConfigFolder) => {
    //           // TODO: Implement show config folder
    //           info!("Show config folder requested");
    //           Ok(())
    //         }
    //         Ok(TrayMenuEvent::ReloadConfig) =>
    // config_reload_tx.send(()),         Ok(TrayMenuEvent::Exit) =>
    // exit_tx.send(()),         Err(err) => {
    //           warn!("Failed to parse tray menu event: {}", err);
    //           continue;
    //         }
    //       };

    //       if let Err(err) = event_res {
    //         warn!("Failed to send tray menu event: {}", err);
    //       }
    //     }

    //     // Small delay to prevent busy-waiting
    //     std::thread::sleep(std::time::Duration::from_millis(10));
    //   }
    // });

    Ok(Self {
      config_reload_rx,
      exit_rx,
      icon_thread: icon_thread_handle,
      tray_icon,
    })
  }

  fn load_icon(path: &Path) -> anyhow::Result<Icon> {
    let (icon_rgba, icon_width, icon_height) = {
      let image = image::open(path)
        .context("Failed to open icon path.")?
        .into_rgba8();

      let (width, height) = image.dimensions();
      let rgba = image.into_raw();
      (rgba, width, height)
    };

    Ok(tray_icon::Icon::from_rgba(
      icon_rgba,
      icon_width,
      icon_height,
    )?)
  }

  /// Destroys the system tray icon and stops its associated message loop.
  pub fn destroy(&mut self) -> anyhow::Result<()> {
    info!("Shutting down system tray.");
    // Tray icon and event thread will be cleaned up when the app exits
    Ok(())
  }
}

impl Drop for SystemTray {
  fn drop(&mut self) {
    if let Err(err) = self.destroy() {
      warn!("Failed to gracefully shut down system tray: {}", err);
    }
  }
}
