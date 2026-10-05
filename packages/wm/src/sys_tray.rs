use std::{
  fmt::{self, Display},
  path::Path,
  str::FromStr,
  sync::{Arc, Mutex},
};

use anyhow::Context;
use tokio::sync::mpsc;
use tray_icon::{
  menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  Icon, TrayIcon, TrayIconBuilder,
};
use wm_common::{InvokeCommand, KeybindingConfig};
use wm_platform::{
  Autostart, Dispatcher, DispatcherExtWindows, Keybinding, ThreadBound,
};

use crate::user_config::UserConfig;

/// Version that the application was built with.
const VERSION: &str = env!("VERSION_NUMBER");

/// Keyboard shortcuts shown alongside the tray menu's actions.
///
/// Only the actions that map to a WM command can have one, since the
/// remaining entries (showing the config folder, toggling animations, and
/// so on) exist solely in the menu.
#[derive(Clone, Debug, Default)]
struct TrayShortcuts {
  rearrange_workspaces: Option<String>,
  reload_config: Option<String>,
  exit: Option<String>,
}

impl TrayShortcuts {
  /// Resolves each action's shortcut from the user's keybindings.
  fn from_config(config: &UserConfig) -> Self {
    Self {
      rearrange_workspaces: shortcut_for(
        &config.value.keybindings,
        &InvokeCommand::WmRearrangeWorkspaces,
      ),
      reload_config: shortcut_for(
        &config.value.keybindings,
        &InvokeCommand::WmReloadConfig,
      ),
      exit: shortcut_for(
        &config.value.keybindings,
        &InvokeCommand::WmExit,
      ),
    }
  }
}

/// The first keybinding bound to the given command, formatted for display.
///
/// Only top-level keybindings are considered, since the ones belonging to
/// a binding mode are unavailable unless that mode is active.
fn shortcut_for(
  keybindings: &[KeybindingConfig],
  command: &InvokeCommand,
) -> Option<String> {
  keybindings
    .iter()
    .find(|keybinding_config| keybinding_config.commands.contains(command))
    .and_then(|keybinding_config| keybinding_config.bindings.first())
    .map(format_shortcut)
}

/// Formats a keybinding the way menus conventionally show shortcuts, for
/// example `Alt+Shift+O`.
fn format_shortcut(keybinding: &Keybinding) -> String {
  keybinding
    .keys()
    .iter()
    .map(|key| {
      let name = key.to_string();
      let mut characters = name.chars();

      match characters.next() {
        Some(first) => {
          first.to_uppercase().chain(characters).collect::<String>()
        }
        None => name,
      }
    })
    .collect::<Vec<_>>()
    .join("+")
}

/// Appends a shortcut to a menu item's label.
///
/// A tab separates the two, which is what the platform's menus use to
/// right-align the shortcut.
fn with_shortcut(label: &str, shortcut: Option<&str>) -> String {
  match shortcut {
    Some(shortcut) => format!("{label}\t{shortcut}"),
    None => label.to_string(),
  }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum TrayMenuId {
  RearrangeWorkspaces,
  ReloadConfig,
  ShowConfigFolder,
  ToggleWindowAnimations,
  RunOnStartup,
  CheckForUpdate,
  Exit,
}

impl Display for TrayMenuId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      TrayMenuId::RearrangeWorkspaces => {
        write!(f, "rearrange_workspaces")
      }
      TrayMenuId::ReloadConfig => write!(f, "reload_config"),
      TrayMenuId::ShowConfigFolder => write!(f, "show_config_folder"),
      TrayMenuId::ToggleWindowAnimations => {
        write!(f, "toggle_window_animations")
      }
      TrayMenuId::RunOnStartup => write!(f, "run_on_startup"),
      TrayMenuId::CheckForUpdate => write!(f, "check_for_update"),
      TrayMenuId::Exit => write!(f, "exit"),
    }
  }
}

impl FromStr for TrayMenuId {
  type Err = anyhow::Error;

  fn from_str(event: &str) -> Result<Self, Self::Err> {
    match event {
      "show_config_folder" => Ok(Self::ShowConfigFolder),
      "rearrange_workspaces" => Ok(Self::RearrangeWorkspaces),
      "reload_config" => Ok(Self::ReloadConfig),
      "toggle_window_animations" => Ok(Self::ToggleWindowAnimations),
      "run_on_startup" => Ok(Self::RunOnStartup),
      "check_for_update" => Ok(Self::CheckForUpdate),
      "exit" => Ok(Self::Exit),
      _ => anyhow::bail!("Invalid tray menu event: {}", event),
    }
  }
}

pub struct SystemTray {
  pub command_rx: mpsc::UnboundedReceiver<InvokeCommand>,
  pub exit_rx: mpsc::UnboundedReceiver<()>,
  animations_enabled: Arc<Mutex<bool>>,
  run_on_startup_enabled: Arc<Mutex<bool>>,
  _icon_thread: Option<std::thread::JoinHandle<()>>,
  tray_icon: ThreadBound<TrayIcon>,
}

impl SystemTray {
  /// Install the system tray on the main thread after the run loop starts.
  pub fn new(
    config: &UserConfig,
    dispatcher: Dispatcher,
  ) -> anyhow::Result<Self> {
    let (exit_tx, exit_rx) = mpsc::unbounded_channel();
    let (command_tx, command_rx) = mpsc::unbounded_channel();

    let animations_enabled = Arc::new(Mutex::new(
      dispatcher.window_animations_enabled().unwrap_or(false),
    ));

    let run_on_startup_enabled =
      Arc::new(Mutex::new(is_run_on_startup_enabled()));

    let shortcuts = TrayShortcuts::from_config(config);

    let tray_icon = dispatcher.dispatch_sync(|| {
      let tray_icon = Self::create_tray_icon(
        *animations_enabled.lock().unwrap(),
        *run_on_startup_enabled.lock().unwrap(),
        &shortcuts,
      )
      .unwrap();
      ThreadBound::new(tray_icon, dispatcher.clone())
    })?;

    // Spawn thread to handle tray menu events.
    let config_path = config.path.clone();
    let thread_animations_enabled = animations_enabled.clone();
    let thread_run_on_startup_enabled = run_on_startup_enabled.clone();

    let icon_thread = std::thread::spawn(move || {
      let menu_event_rx = MenuEvent::receiver();

      while let Ok(event) = menu_event_rx.recv() {
        if let Ok(menu_event) = TrayMenuId::from_str(event.id.as_ref()) {
          if let Err(err) = Self::handle_menu_event(
            &menu_event,
            &dispatcher,
            &config_path,
            &command_tx,
            &exit_tx,
            &thread_animations_enabled,
            &thread_run_on_startup_enabled,
          ) {
            tracing::warn!("Failed to handle tray menu event: {}", err);
          }
        }
      }
    });

    Ok(Self {
      command_rx,
      exit_rx,
      animations_enabled,
      run_on_startup_enabled,
      _icon_thread: Some(icon_thread),
      tray_icon,
    })
  }

  /// Rebuilds the tray menu so that the shortcuts it shows match the
  /// user's current keybindings.
  ///
  /// Called after the config is reloaded, since the menu is otherwise
  /// only built once at startup.
  pub fn update_menu(&self, config: &UserConfig) -> anyhow::Result<()> {
    let shortcuts = TrayShortcuts::from_config(config);
    let animations_enabled = *self.animations_enabled.lock().unwrap();
    let run_on_startup_enabled =
      *self.run_on_startup_enabled.lock().unwrap();

    // The menu has to be built on the thread that owns the tray icon.
    self.tray_icon.with(move |tray_icon| {
      let menu = Self::create_menu(
        animations_enabled,
        run_on_startup_enabled,
        &shortcuts,
      )?;

      tray_icon.set_menu(Some(Box::new(menu)));
      anyhow::Ok(())
    })??;

    Ok(())
  }

  /// Builds the tray's context menu.
  ///
  /// Actions that the user has a keybinding for show it alongside their
  /// label, the way menu shortcuts are conventionally displayed.
  fn create_menu(
    animations_enabled: bool,
    run_on_startup_enabled: bool,
    shortcuts: &TrayShortcuts,
  ) -> anyhow::Result<Menu> {
    // Disabled so that it reads as a heading rather than an action.
    let version_item =
      MenuItem::new(format!("GlazeWM v{VERSION}"), false, None);

    let rearrange_workspaces_item = MenuItem::with_id(
      TrayMenuId::RearrangeWorkspaces,
      with_shortcut(
        "Rearrange workspaces",
        shortcuts.rearrange_workspaces.as_deref(),
      ),
      true,
      None,
    );

    let reload_config_item = MenuItem::with_id(
      TrayMenuId::ReloadConfig,
      with_shortcut("Reload config", shortcuts.reload_config.as_deref()),
      true,
      None,
    );

    let config_dir_item = MenuItem::with_id(
      TrayMenuId::ShowConfigFolder,
      "Show config folder",
      true,
      None,
    );

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

    let check_for_update_item = MenuItem::with_id(
      TrayMenuId::CheckForUpdate,
      "Check for updates",
      true,
      None,
    );

    let exit_item = MenuItem::with_id(
      TrayMenuId::Exit,
      with_shortcut("Exit", shortcuts.exit.as_deref()),
      true,
      None,
    );

    let tray_menu = Menu::new();
    tray_menu.append_items(&[
      &version_item,
      &PredefinedMenuItem::separator(),
      &rearrange_workspaces_item,
      &reload_config_item,
      &config_dir_item,
      &toggle_animations_item,
      &run_on_startup_item,
      &PredefinedMenuItem::separator(),
      &check_for_update_item,
      &exit_item,
    ])?;

    Ok(tray_menu)
  }

  fn create_tray_icon(
    animations_enabled: bool,
    run_on_startup_enabled: bool,
    shortcuts: &TrayShortcuts,
  ) -> anyhow::Result<TrayIcon> {
    let tray_menu = Self::create_menu(
      animations_enabled,
      run_on_startup_enabled,
      shortcuts,
    )?;

    let icon = Self::load_icon(include_bytes!(
      "../../../resources/assets/icon.png"
    ))?;

    let tray_icon = TrayIconBuilder::new()
      .with_menu(Box::new(tray_menu))
      .with_tooltip(format!("GlazeWM v{VERSION}"))
      .with_icon(icon)
      .build()?;

    Ok(tray_icon)
  }

  fn load_icon(bytes: &[u8]) -> anyhow::Result<Icon> {
    let (icon_rgba, icon_width, icon_height) = {
      let image = image::load_from_memory(bytes)
        .context("Failed to to create tray icon image from resource.")?
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

  fn handle_menu_event(
    menu_id: &TrayMenuId,
    dispatcher: &Dispatcher,
    config_path: &Path,
    command_tx: &mpsc::UnboundedSender<InvokeCommand>,
    exit_tx: &mpsc::UnboundedSender<()>,
    animations_enabled: &Arc<Mutex<bool>>,
    run_on_startup_enabled: &Arc<Mutex<bool>>,
  ) -> anyhow::Result<()> {
    tracing::info!("Processing tray menu event: {:?}", menu_id);

    match menu_id {
      TrayMenuId::ShowConfigFolder => {
        dispatcher.open_file_explorer(
          config_path.parent().context("Invalid config path.")?,
        )?;

        Ok(())
      }
      TrayMenuId::ReloadConfig => {
        command_tx.send(InvokeCommand::WmReloadConfig)?;
        Ok(())
      }
      TrayMenuId::RearrangeWorkspaces => {
        command_tx.send(InvokeCommand::WmRearrangeWorkspaces)?;
        Ok(())
      }
      TrayMenuId::ToggleWindowAnimations => {
        let mut animations_enabled = animations_enabled.lock().unwrap();
        dispatcher.set_window_animations_enabled(!*animations_enabled)?;
        *animations_enabled = !*animations_enabled;
        Ok(())
      }
      TrayMenuId::RunOnStartup => {
        let mut run_on_startup_enabled =
          run_on_startup_enabled.lock().unwrap();

        if *run_on_startup_enabled {
          autostart_instance()?.disable()?;
        } else {
          autostart_instance()?.enable()?;
        }

        *run_on_startup_enabled = !*run_on_startup_enabled;
        Ok(())
      }
      TrayMenuId::CheckForUpdate => {
        crate::updater::check_for_updates(dispatcher, exit_tx);
        Ok(())
      }
      TrayMenuId::Exit => {
        exit_tx.send(())?;
        Ok(())
      }
    }
  }
}

/// Creates an [`Autostart`] for starting the WM when the user signs in.
fn autostart_instance() -> anyhow::Result<Autostart> {
  let exe_path = std::env::current_exe()?;
  Ok(Autostart::new("GlazeWM", &exe_path)?)
}

/// Whether the WM is registered to start when the user signs in.
///
/// Earlier versions registered under the `Run` registry key, which is
/// carried over first so that the setting survives the change of
/// mechanism. Failures are logged and reported as not enabled, since the
/// tray should come up regardless.
fn is_run_on_startup_enabled() -> bool {
  let autostart = match autostart_instance() {
    Ok(autostart) => autostart,
    Err(err) => {
      tracing::warn!("Failed to resolve autostart registration: {}", err);
      return false;
    }
  };

  match autostart.migrate_from_run_key() {
    Ok(true) => {
      tracing::info!("Moved autostart registration to a scheduled task.");
    }
    Ok(false) => {}
    Err(err) => {
      tracing::warn!("Failed to migrate autostart registration: {}", err);
    }
  }

  autostart.is_enabled().unwrap_or_else(|err| {
    tracing::warn!("Failed to query autostart registration: {}", err);
    false
  })
}

#[cfg(test)]
mod tests {
  use wm_common::{InvokeCommand, KeybindingConfig};
  use wm_platform::{Key, Keybinding};

  use super::{format_shortcut, shortcut_for, with_shortcut};

  fn binding(
    keys: Vec<Key>,
    commands: Vec<InvokeCommand>,
  ) -> KeybindingConfig {
    KeybindingConfig {
      bindings: vec![Keybinding::new(keys).unwrap()],
      commands,
    }
  }

  #[test]
  fn finds_the_shortcut_bound_to_a_command() {
    let keybindings = vec![
      binding(
        vec![Key::Alt, Key::Shift, Key::R],
        vec![InvokeCommand::WmReloadConfig],
      ),
      binding(
        vec![Key::Alt, Key::Shift, Key::O],
        vec![InvokeCommand::WmRearrangeWorkspaces],
      ),
    ];

    assert_eq!(
      shortcut_for(&keybindings, &InvokeCommand::WmRearrangeWorkspaces),
      Some("Alt+Shift+O".to_string())
    );
  }

  #[test]
  fn has_no_shortcut_for_an_unbound_command() {
    let keybindings = vec![binding(
      vec![Key::Alt, Key::Shift, Key::R],
      vec![InvokeCommand::WmReloadConfig],
    )];

    assert_eq!(shortcut_for(&keybindings, &InvokeCommand::WmExit), None);
  }

  #[test]
  fn finds_a_command_bound_alongside_others() {
    let keybindings = vec![binding(
      vec![Key::Alt, Key::Shift, Key::O],
      vec![
        InvokeCommand::ToggleTiling,
        InvokeCommand::WmRearrangeWorkspaces,
      ],
    )];

    assert_eq!(
      shortcut_for(&keybindings, &InvokeCommand::WmRearrangeWorkspaces),
      Some("Alt+Shift+O".to_string())
    );
  }

  #[test]
  fn formats_a_keybinding_for_a_menu() {
    let keybinding =
      Keybinding::new(vec![Key::Alt, Key::Shift, Key::O]).unwrap();

    assert_eq!(format_shortcut(&keybinding), "Alt+Shift+O");
  }

  #[test]
  fn formats_single_key_and_named_keys() {
    let single = Keybinding::new(vec![Key::F1]).unwrap();
    assert_eq!(format_shortcut(&single), "F1");

    let named = Keybinding::new(vec![Key::Alt, Key::Left]).unwrap();
    assert_eq!(format_shortcut(&named), "Alt+Left");
  }

  #[test]
  fn separates_the_shortcut_from_the_label_with_a_tab() {
    assert_eq!(
      with_shortcut("Reload config", Some("Alt+Shift+R")),
      "Reload config\tAlt+Shift+R"
    );
  }

  #[test]
  fn leaves_the_label_alone_without_a_shortcut() {
    assert_eq!(with_shortcut("Exit", None), "Exit");
  }
}
