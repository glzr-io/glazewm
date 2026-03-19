use std::{
  collections::HashMap,
  fmt::{self, Display},
  path::Path,
  str::FromStr,
  sync::{Arc, Mutex},
};

use anyhow::Context;
use auto_launch::AutoLaunch;
use image::{Rgba, RgbaImage};
use tokio::sync::mpsc;
use tray_icon::{
  menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  Icon, TrayIcon, TrayIconBuilder,
};
use wm_common::{TilingDirection, TrayIconMode};
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DisplayedTrayIcon {
  Status(TrayIconState),
  Workspace(u8),
}

struct TrayIconSet {
  static_icon_image: RgbaImage,
  static_icon: Icon,
  disabled_icon: Icon,
  horizontal_icon: Icon,
  vertical_icon: Icon,
}

impl TrayIconSet {
  /// Creates the full set of tray icons from bundled resources.
  fn new() -> anyhow::Result<Self> {
    let static_icon_image = SystemTray::load_icon_image(include_bytes!(
      "../../../resources/assets/icon.png"
    ))?;

    Ok(Self {
      static_icon: SystemTray::icon_from_image(&static_icon_image)?,
      static_icon_image,
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

struct WorkspaceIconCache {
  icons: HashMap<u8, Icon>,
}

impl WorkspaceIconCache {
  /// Creates a new workspace tray icon cache.
  fn new() -> Self {
    Self {
      icons: HashMap::new(),
    }
  }

  /// Gets a cached workspace icon, creating it if necessary.
  fn icon(
    &mut self,
    static_icon_image: &RgbaImage,
    workspace_index: usize,
  ) -> anyhow::Result<Icon> {
    let workspace_index = clamp_workspace_index(workspace_index);

    if let Some(icon) = self.icons.get(&workspace_index) {
      return Ok(icon.clone());
    }

    let icon_image =
      render_workspace_icon_image(static_icon_image, workspace_index);
    let icon = SystemTray::icon_from_image(&icon_image)?;

    self.icons.insert(workspace_index, icon.clone());

    Ok(icon)
  }
}

struct TrayIconHandle {
  tray_icon: TrayIcon,
  icons: TrayIconSet,
  workspace_icons: Mutex<WorkspaceIconCache>,
}

impl TrayIconHandle {
  /// Updates the currently shown tray icon.
  fn set_icon_state(
    &self,
    state: DisplayedTrayIcon,
  ) -> anyhow::Result<()> {
    let icon = match state {
      DisplayedTrayIcon::Status(state) => self.icons.icon(state).clone(),
      DisplayedTrayIcon::Workspace(workspace_index) => self
        .workspace_icons
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .icon(
          &self.icons.static_icon_image,
          usize::from(workspace_index),
        )?,
    };

    self.tray_icon.set_icon(Some(icon))?;
    Ok(())
  }
}

pub struct SystemTray {
  pub config_reload_rx: mpsc::UnboundedReceiver<()>,
  pub exit_rx: mpsc::UnboundedReceiver<()>,
  _icon_thread: Option<std::thread::JoinHandle<()>>,
  tray_icon: ThreadBound<TrayIconHandle>,
  current_icon_state: DisplayedTrayIcon,
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
            workspace_icons: Mutex::new(WorkspaceIconCache::new()),
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
      current_icon_state: DisplayedTrayIcon::Status(TrayIconState::Static),
    })
  }

  /// Synchronizes the tray icon with the current WM status.
  pub fn sync_status(
    &mut self,
    tray_icon_mode: TrayIconMode,
    tray_icon_state_enabled: bool,
    is_paused: bool,
    tiling_direction: Option<&TilingDirection>,
    workspace_index: Option<usize>,
  ) -> anyhow::Result<()> {
    let target_state = target_displayed_icon(
      tray_icon_mode,
      tray_icon_state_enabled,
      is_paused,
      tiling_direction,
      workspace_index,
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

  /// Loads a tray icon image from bundled bytes.
  fn load_icon_image(bytes: &[u8]) -> anyhow::Result<RgbaImage> {
    image::load_from_memory(bytes)
      .context("Failed to create tray icon image from resource.")
      .map(|image| image.into_rgba8())
  }

  /// Loads a tray icon from bundled bytes.
  fn load_icon(bytes: &[u8]) -> anyhow::Result<Icon> {
    let image = Self::load_icon_image(bytes)?;

    Self::icon_from_image(&image)
  }

  /// Creates a tray icon from an RGBA image.
  fn icon_from_image(image: &RgbaImage) -> anyhow::Result<Icon> {
    let (width, height) = image.dimensions();
    Ok(tray_icon::Icon::from_rgba(
      image.as_raw().clone(),
      width,
      height,
    )?)
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

/// Determines which tray icon should be shown for the current WM state.
fn target_displayed_icon(
  tray_icon_mode: TrayIconMode,
  tray_icon_state_enabled: bool,
  is_paused: bool,
  tiling_direction: Option<&TilingDirection>,
  workspace_index: Option<usize>,
) -> DisplayedTrayIcon {
  match tray_icon_mode {
    TrayIconMode::Status => DisplayedTrayIcon::Status(target_icon_state(
      tray_icon_state_enabled,
      is_paused,
      tiling_direction,
    )),
    TrayIconMode::Workspace => workspace_index
      .map(clamp_workspace_index)
      .map(DisplayedTrayIcon::Workspace)
      .unwrap_or(DisplayedTrayIcon::Status(TrayIconState::Static)),
  }
}

/// Clamps a workspace index to the supported badge range.
fn clamp_workspace_index(workspace_index: usize) -> u8 {
  if workspace_index > 99 {
    99
  } else {
    workspace_index as u8
  }
}

/// Renders a workspace badge on top of the base tray icon.
fn render_workspace_icon_image(
  static_icon_image: &RgbaImage,
  workspace_index: u8,
) -> RgbaImage {
  let digits = workspace_digits(workspace_index);

  let (digit_width, digit_height, segment_width, digit_gap, _padding_x) =
    match digits.len() {
      1 => (126, 224, 24, 0, 22),
      _ => (92, 208, 20, 12, 18),
    };

  let mut image = static_icon_image.clone();
  let image_width = image.width();
  let image_height = image.height();
  let digits_width = digit_width * digits.len() as u32
    + digit_gap * digits.len().saturating_sub(1) as u32;
  let digit_top = (static_icon_image.height() - digit_height) / 2;
  let mut digit_left = (static_icon_image.width() - digits_width) / 2;

  darken_rect(&mut image, 0, 0, image_width, image_height, 3, 5);

  for digit in digits {
    draw_workspace_digit(
      &mut image,
      digit,
      digit_left,
      digit_top,
      digit_width,
      digit_height,
      segment_width,
      Rgba([255, 255, 255, 255]),
    );
    digit_left += digit_width + digit_gap;
  }

  image
}

/// Converts a workspace index into one or two display digits.
fn workspace_digits(workspace_index: u8) -> Vec<u8> {
  if workspace_index >= 10 {
    vec![workspace_index / 10, workspace_index % 10]
  } else {
    vec![workspace_index]
  }
}

/// Draws a single workspace digit.
fn draw_workspace_digit(
  image: &mut RgbaImage,
  digit: u8,
  left: u32,
  top: u32,
  width: u32,
  height: u32,
  segment_width: u32,
  color: Rgba<u8>,
) {
  let mid_y = top + height / 2 - segment_width / 2;
  let upper_height = height / 2 - segment_width;
  let lower_height = height / 2 - segment_width;
  let horizontal_width = width.saturating_sub(segment_width * 2);

  let segments = digit_segments(digit);

  if segments[0] {
    draw_rect(
      image,
      left + segment_width,
      top,
      horizontal_width,
      segment_width,
      color,
    );
  }
  if segments[1] {
    draw_rect(
      image,
      left + width - segment_width,
      top + segment_width,
      segment_width,
      upper_height,
      color,
    );
  }
  if segments[2] {
    draw_rect(
      image,
      left + width - segment_width,
      mid_y + segment_width,
      segment_width,
      lower_height,
      color,
    );
  }
  if segments[3] {
    draw_rect(
      image,
      left + segment_width,
      top + height - segment_width,
      horizontal_width,
      segment_width,
      color,
    );
  }
  if segments[4] {
    draw_rect(
      image,
      left,
      mid_y + segment_width,
      segment_width,
      lower_height,
      color,
    );
  }
  if segments[5] {
    draw_rect(
      image,
      left,
      top + segment_width,
      segment_width,
      upper_height,
      color,
    );
  }
  if segments[6] {
    draw_rect(
      image,
      left + segment_width,
      mid_y,
      horizontal_width,
      segment_width,
      color,
    );
  }
}

/// Gets the active seven-segment mask for a digit.
fn digit_segments(digit: u8) -> [bool; 7] {
  match digit {
    0 => [true, true, true, true, true, true, false],
    1 => [false, true, true, false, false, false, false],
    2 => [true, true, false, true, true, false, true],
    3 => [true, true, true, true, false, false, true],
    4 => [false, true, true, false, false, true, true],
    5 => [true, false, true, true, false, true, true],
    6 => [true, false, true, true, true, true, true],
    7 => [true, true, true, false, false, false, false],
    8 => [true, true, true, true, true, true, true],
    9 => [true, true, true, true, false, true, true],
    _ => [false; 7],
  }
}

/// Draws a filled rectangle.
fn draw_rect(
  image: &mut RgbaImage,
  left: u32,
  top: u32,
  width: u32,
  height: u32,
  color: Rgba<u8>,
) {
  let right = (left + width).min(image.width());
  let bottom = (top + height).min(image.height());

  for y in top..bottom {
    for x in left..right {
      image.put_pixel(x, y, color);
    }
  }
}

/// Darkens a rectangular region by a fixed ratio.
fn darken_rect(
  image: &mut RgbaImage,
  left: u32,
  top: u32,
  width: u32,
  height: u32,
  numerator: u16,
  denominator: u16,
) {
  let right = (left + width).min(image.width());
  let bottom = (top + height).min(image.height());

  for y in top..bottom {
    for x in left..right {
      let pixel = image.get_pixel_mut(x, y);

      for channel in &mut pixel.0[..3] {
        *channel = ((u16::from(*channel) * numerator) / denominator) as u8;
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use wm_common::{TilingDirection, TrayIconMode};

  use super::{
    clamp_workspace_index, render_workspace_icon_image,
    target_displayed_icon, target_icon_state, DisplayedTrayIcon,
    TrayIconState, WorkspaceIconCache,
  };

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

  #[test]
  fn target_displayed_icon_uses_status_mode_mapping() {
    let state = target_displayed_icon(
      TrayIconMode::Status,
      true,
      false,
      Some(&TilingDirection::Horizontal),
      Some(7),
    );

    assert_eq!(
      state,
      DisplayedTrayIcon::Status(TrayIconState::Horizontal)
    );
  }

  #[test]
  fn target_displayed_icon_uses_workspace_icon_in_workspace_mode() {
    let state = target_displayed_icon(
      TrayIconMode::Workspace,
      false,
      true,
      Some(&TilingDirection::Vertical),
      Some(7),
    );

    assert_eq!(state, DisplayedTrayIcon::Workspace(7));
  }

  #[test]
  fn target_displayed_icon_falls_back_without_workspace_index() {
    let state = target_displayed_icon(
      TrayIconMode::Workspace,
      true,
      false,
      None,
      None,
    );

    assert_eq!(state, DisplayedTrayIcon::Status(TrayIconState::Static));
  }

  #[test]
  fn clamp_workspace_index_caps_values_at_99() {
    assert_eq!(clamp_workspace_index(7), 7);
    assert_eq!(clamp_workspace_index(99), 99);
    assert_eq!(clamp_workspace_index(100), 99);
  }

  #[test]
  fn render_workspace_icon_image_supports_single_and_double_digits() {
    let base_icon = super::SystemTray::load_icon_image(include_bytes!(
      "../../../resources/assets/icon.png"
    ))
    .expect("Failed to load tray icon image.");

    let single_digit = render_workspace_icon_image(&base_icon, 7);
    let double_digit = render_workspace_icon_image(&base_icon, 12);

    assert_eq!(single_digit.dimensions(), (256, 256));
    assert_eq!(double_digit.dimensions(), (256, 256));
    assert_ne!(single_digit.as_raw(), double_digit.as_raw());
  }

  #[test]
  fn workspace_icon_cache_reuses_clamped_entries() {
    let base_icon = super::SystemTray::load_icon_image(include_bytes!(
      "../../../resources/assets/icon.png"
    ))
    .expect("Failed to load tray icon image.");
    let mut cache = WorkspaceIconCache::new();

    let _ = cache
      .icon(&base_icon, 100)
      .expect("Failed to render first workspace icon.");
    let _ = cache
      .icon(&base_icon, 99)
      .expect("Failed to render second workspace icon.");

    assert_eq!(cache.icons.len(), 1);
  }
}
