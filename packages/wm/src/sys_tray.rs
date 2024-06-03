use anyhow::Context;
use tokio::sync::oneshot;
use tray_icon::{
  menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
  TrayIconBuilder,
};

use crate::common::platform::Platform;

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

  let menu_channel = MenuEvent::receiver();

  std::thread::spawn(move || {
    menu_channel.iter().for_each(|m| match m {
      event => {
        println!("{:?}", event);
      }
    })
  });

  let tray_icon = TrayIconBuilder::new()
    .with_menu(Box::new(tray_menu))
    .with_tooltip("test")
    .with_icon(load_icon()?)
    .build()?;

  let (abort_tx, abort_rx) = oneshot::channel();
  Platform::run_message_loop(abort_rx);

  Ok(())
}

fn load_icon() -> anyhow::Result<tray_icon::Icon> {
  let icon = include_bytes!("../../../resources/icon.ico");

  let (icon_rgba, icon_width, icon_height) = {
    let image = image::load_from_memory(icon)
      .context("Failed to open icon path.")?
      .into_rgba8();

    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    (rgba, width, height)
  };

  tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height)
    .context("Failed to open icon.")
}
