use std::{
  fmt::{self, Display},
  path::Path,
  str::FromStr,
  sync::{Arc, Mutex},
};

use anyhow::Context;
use auto_launch::AutoLaunch;
use tokio::sync::mpsc;
use tray_icon::{
  menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  Icon, TrayIcon, TrayIconBuilder,
};
use wm_common::TilingDirection;
#[cfg(target_os = "windows")]
use wm_platform::DispatcherExtWindows;
use wm_platform::{Dispatcher, ThreadBound};

#[derive(Debug, Clone, Eq, PartialEq)]
enum TrayMenuId {
  ReloadConfig,
  ShowConfigFolder,
  #[cfg(target_os = "windows")]
  ToggleWindowAnimations,
  RunOnStartup,
  Exit,
}

impl Display for TrayMenuId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      TrayMenuId::ReloadConfig => write!(f, "reload_config"),
      TrayMenuId::ShowConfigFolder => write!(f, "show_config_folder"),
      #[cfg(target_os = "windows")]
      TrayMenuId::ToggleWindowAnimations => {
        write!(f, "toggle_window_animations")
      }
      TrayMenuId::RunOnStartup => write!(f, "run_on_startup"),
      TrayMenuId::Exit => write!(f, "exit"),
    }
  }
}

impl FromStr for TrayMenuId {
  type Err = anyhow::Error;

  fn from_str(event: &str) -> Result<Self, Self::Err> {
    match event {
      "show_config_folder" => Ok(Self::ShowConfigFolder),
      "reload_config" => Ok(Self::ReloadConfig),
      #[cfg(target_os = "windows")]
      "toggle_window_animations" => Ok(Self::ToggleWindowAnimations),
      "run_on_startup" => Ok(Self::RunOnStartup),
      "exit" => Ok(Self::Exit),
      _ => anyhow::bail!("Invalid tray menu event: {}", event),
    }
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TrayIconState {
  Static,
  Disabled,
  Horizontal,
  Vertical,
}

struct TrayIconSet {
  static_icon: Icon,
  disabled_icon: Icon,
  horizontal_icon: Icon,
  vertical_icon: Icon,
}

impl TrayIconSet {
  /// Creates the full set of tray icons from bundled resources.
  fn new() -> anyhow::Result<Self> {
    Ok(Self {
      static_icon: SystemTray::load_icon(include_bytes!(
        "../../../resources/assets/icon.png"
      ))?,
      disabled_icon: SystemTray::load_icon(include_bytes!(
        "../../../resources/assets/icon_disabled.png"
      ))?,
      horizontal_icon: SystemTray::load_icon(include_bytes!(
        "../../../resources/assets/icon_horizontal.png"
      ))?,
      vertical_icon: SystemTray::load_icon(include_bytes!(
        "../../../resources/assets/icon_vertical.png"
      ))?,
    })
  }

  /// Gets the icon corresponding to the given tray status.
  fn icon(&self, state: TrayIconState) -> &Icon {
    match state {
      TrayIconState::Static => &self.static_icon,
      TrayIconState::Disabled => &self.disabled_icon,
      TrayIconState::Horizontal => &self.horizontal_icon,
      TrayIconState::Vertical => &self.vertical_icon,
    }
  }
}

struct TrayIconHandle {
  tray_icon: TrayIcon,
  icons: TrayIconSet,
}

impl TrayIconHandle {
  /// Updates the currently shown tray icon.
  fn set_icon_state(&self, state: TrayIconState) -> anyhow::Result<()> {
    self
      .tray_icon
      .set_icon(Some(self.icons.icon(state).clone()))?;
    Ok(())
  }
}

pub struct SystemTray {
  pub config_reload_rx: mpsc::UnboundedReceiver<()>,
  pub exit_rx: mpsc::UnboundedReceiver<()>,
  _icon_thread: Option<std::thread::JoinHandle<()>>,
  tray_icon: ThreadBound<TrayIconHandle>,
  current_icon_state: TrayIconState,
}

impl SystemTray {
  /// Install the system tray on the main thread after the run loop starts.
  pub fn new(
    config_path: &Path,
    dispatcher: Dispatcher,
  ) -> anyhow::Result<Self> {
    let (exit_tx, exit_rx) = mpsc::unbounded_channel();
    let (config_reload_tx, config_reload_rx) = mpsc::unbounded_channel();

    let animations_enabled = Arc::new(Mutex::new({
      #[cfg(target_os = "windows")]
      {
        dispatcher.window_animations_enabled().unwrap_or(false)
      }
      #[cfg(not(target_os = "windows"))]
      {
        false
      }
    }));

    let run_on_startup_enabled = Arc::new(Mutex::new(
      auto_launch_instance()
        .and_then(|auto_launch| {
          auto_launch.is_enabled().map_err(Into::into)
        })
        .unwrap_or(false),
    ));

    let tray_icon =
      dispatcher.dispatch_sync(|| -> anyhow::Result<_> {
        let tray_icons = TrayIconSet::new()?;
        let tray_icon = Self::create_tray_icon(
          *animations_enabled
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
          *run_on_startup_enabled
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
          tray_icons.icon(TrayIconState::Static).clone(),
        )?;

        Ok(ThreadBound::new(
          TrayIconHandle {
            tray_icon,
            icons: tray_icons,
          },
          dispatcher.clone(),
        ))
      })??;

    // Spawn thread to handle tray menu events.
    let config_path = config_path.to_owned();
    let icon_thread = std::thread::spawn(move || {
      let menu_event_rx = MenuEvent::receiver();

      while let Ok(event) = menu_event_rx.recv() {
        if let Ok(menu_event) = TrayMenuId::from_str(event.id.as_ref()) {
          if let Err(err) = Self::handle_menu_event(
            &menu_event,
            &dispatcher,
            &config_path,
            &config_reload_tx,
            &exit_tx,
            &animations_enabled,
            &run_on_startup_enabled,
          ) {
            tracing::warn!("Failed to handle tray menu event: {}", err);
          }
        }
      }
    });

    Ok(Self {
      config_reload_rx,
      exit_rx,
      _icon_thread: Some(icon_thread),
      tray_icon,
      current_icon_state: TrayIconState::Static,
    })
  }

  /// Synchronizes the tray icon with the current WM status.
  pub fn sync_status(
    &mut self,
    tray_icon_state_enabled: bool,
    is_paused: bool,
    tiling_direction: Option<&TilingDirection>,
  ) -> anyhow::Result<()> {
    let target_state = target_icon_state(
      tray_icon_state_enabled,
      is_paused,
      tiling_direction,
    );

    if target_state == self.current_icon_state {
      return Ok(());
    }

    self
      .tray_icon
      .with(|tray_icon| tray_icon.set_icon_state(target_state))??;
    self.current_icon_state = target_state;

    Ok(())
  }

  /// Creates the tray icon and its context menu.
  fn create_tray_icon(
    // LINT: `animations_enabled` is only used on Windows.
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    animations_enabled: bool,
    run_on_startup_enabled: bool,
    icon: Icon,
  ) -> anyhow::Result<TrayIcon> {
    let reload_config_item = MenuItem::with_id(
      TrayMenuId::ReloadConfig,
      "Reload config",
      true,
      None,
    );

    let config_dir_item = MenuItem::with_id(
      TrayMenuId::ShowConfigFolder,
      "Show config folder",
      true,
      None,
    );

    #[cfg(target_os = "windows")]
    let toggle_animations_item = CheckMenuItem::with_id(
      TrayMenuId::ToggleWindowAnimations,
      "Window animations",
      true,
      animations_enabled,
      None,
    );

    let run_on_startup_item = CheckMenuItem::with_id(
      TrayMenuId::RunOnStartup,
      "Run on system startup",
      true,
      run_on_startup_enabled,
      None,
    );

    let exit_item =
      MenuItem::with_id(TrayMenuId::Exit, "Exit", true, None);

    let tray_menu = Menu::new();
    tray_menu.append_items(&[
      &reload_config_item,
      &config_dir_item,
      #[cfg(target_os = "windows")]
      &toggle_animations_item,
      &run_on_startup_item,
      &PredefinedMenuItem::separator(),
      &exit_item,
    ])?;

    let tray_icon = TrayIconBuilder::new()
      .with_menu(Box::new(tray_menu))
      .with_tooltip(format!("GlazeWM v{}", env!("VERSION_NUMBER")))
      .with_icon(icon)
      .build()?;

    Ok(tray_icon)
  }

  /// Loads a tray icon from bundled bytes.
  fn load_icon(bytes: &[u8]) -> anyhow::Result<Icon> {
    let image = image::load_from_memory(bytes)
      .context("Failed to create tray icon image from resource.")?
      .into_rgba8();
    let (width, height) = image.dimensions();

    Ok(tray_icon::Icon::from_rgba(image.into_raw(), width, height)?)
  }

  /// Handles a tray menu event.
  fn handle_menu_event(
    menu_id: &TrayMenuId,
    dispatcher: &Dispatcher,
    config_path: &Path,
    config_reload_tx: &mpsc::UnboundedSender<()>,
    exit_tx: &mpsc::UnboundedSender<()>,
    // LINT: `animations_enabled` is only used on Windows.
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    animations_enabled: &Arc<Mutex<bool>>,
    run_on_startup_enabled: &Arc<Mutex<bool>>,
  ) -> anyhow::Result<()> {
    tracing::info!("Processing tray menu event: {:?}", menu_id);

    match menu_id {
      TrayMenuId::ShowConfigFolder => {
        dispatcher.open_file_explorer(config_path)?;
        Ok(())
      }
      TrayMenuId::ReloadConfig => {
        config_reload_tx.send(())?;
        Ok(())
      }
      #[cfg(target_os = "windows")]
      TrayMenuId::ToggleWindowAnimations => {
        let mut animations_enabled = animations_enabled
          .lock()
          .unwrap_or_else(|poisoned| poisoned.into_inner());
        dispatcher.set_window_animations_enabled(!*animations_enabled)?;
        *animations_enabled = !*animations_enabled;
        Ok(())
      }
      TrayMenuId::RunOnStartup => {
        let mut run_on_startup_enabled = run_on_startup_enabled
          .lock()
          .unwrap_or_else(|poisoned| poisoned.into_inner());

        if *run_on_startup_enabled {
          auto_launch_instance()?.disable()?;
        } else {
          auto_launch_instance()?.enable()?;
        }

        *run_on_startup_enabled = !*run_on_startup_enabled;
        Ok(())
      }
      TrayMenuId::Exit => {
        exit_tx.send(())?;
        Ok(())
      }
    }
  }
}

/// Creates a new [`AutoLaunch`] instance for managing auto-launch at
/// system startup.
fn auto_launch_instance() -> anyhow::Result<AutoLaunch> {
  // TODO: Is wrapping the exe path in quotes necessary?
  let formatted_exe_path =
    format!("\"{}\"", std::env::current_exe()?.to_string_lossy());
  let args: [&str; 0] = [];

  #[cfg(target_os = "windows")]
  let instance = AutoLaunch::new("GlazeWM", &formatted_exe_path, &args);

  #[cfg(target_os = "macos")]
  let instance =
    AutoLaunch::new("GlazeWM", &formatted_exe_path, false, &args);

  Ok(instance)
}

/// Determines which tray icon should be shown for the current WM state.
fn target_icon_state(
  tray_icon_state_enabled: bool,
  is_paused: bool,
  tiling_direction: Option<&TilingDirection>,
) -> TrayIconState {
  if !tray_icon_state_enabled {
    return TrayIconState::Static;
  }

  if is_paused {
    return TrayIconState::Disabled;
  }

  match tiling_direction {
    Some(TilingDirection::Horizontal) => TrayIconState::Horizontal,
    Some(TilingDirection::Vertical) => TrayIconState::Vertical,
    None => TrayIconState::Static,
  }
}

#[cfg(test)]
mod tests {
  use wm_common::TilingDirection;

  use super::{target_icon_state, TrayIconState};

  #[test]
  fn target_icon_state_uses_static_icon_when_feature_disabled() {
    let state =
      target_icon_state(false, false, Some(&TilingDirection::Horizontal));

    assert_eq!(state, TrayIconState::Static);
  }

  #[test]
  fn target_icon_state_uses_disabled_icon_when_paused() {
    let state =
      target_icon_state(true, true, Some(&TilingDirection::Horizontal));

    assert_eq!(state, TrayIconState::Disabled);
  }

  #[test]
  fn target_icon_state_uses_horizontal_icon_for_horizontal_layout() {
    let state =
      target_icon_state(true, false, Some(&TilingDirection::Horizontal));

    assert_eq!(state, TrayIconState::Horizontal);
  }

  #[test]
  fn target_icon_state_uses_vertical_icon_for_vertical_layout() {
    let state =
      target_icon_state(true, false, Some(&TilingDirection::Vertical));

    assert_eq!(state, TrayIconState::Vertical);
  }

  #[test]
  fn target_icon_state_falls_back_to_static_without_direction() {
    let state = target_icon_state(true, false, None);

    assert_eq!(state, TrayIconState::Static);
  }
}
