use tokio::sync::oneshot;
use tray_icon::{
  menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  Icon, TrayIconBuilder,
};

use crate::common::platform::Platform;

/// Ordinal to IDI_ICON definition from `resources.rc`.
const IDI_ICON: u16 = 0x101;

pub fn add_tray() -> anyhow::Result<()> {
  let tray_menu = Menu::new();

  let quit_item = MenuItem::new("Quit", true, None);

  tray_menu.append_items(&[
    &PredefinedMenuItem::about(
      None,
      Some(AboutMetadata {
        name: Some("test".to_string()),
        copyright: Some("Copyright test".to_string()),
        ..Default::default()
      }),
    ),
    &PredefinedMenuItem::separator(),
    &quit_item,
  ])?;

  let menu_event_rx = MenuEvent::receiver();

  std::thread::spawn(move || {
    menu_event_rx.iter().for_each(|m| match m {
      event => {
        println!("{:?}", event);
      }
    })
  });

  let icon = Icon::from_resource(IDI_ICON, None)?;
  let tray_icon = TrayIconBuilder::new()
    .with_menu(Box::new(tray_menu))
    .with_tooltip("test")
    .with_icon(icon)
    .build()?;

  let (abort_tx, abort_rx) = oneshot::channel();
  Platform::run_message_loop(abort_rx);

  Ok(())
}
