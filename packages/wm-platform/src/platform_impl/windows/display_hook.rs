use std::sync::OnceLock;

use windows::Win32::{
  Foundation::WPARAM,
  UI::WindowsAndMessaging::{
    DBT_DEVNODES_CHANGED, SPI_ICONVERTICALSPACING, SPI_SETWORKAREA,
    WM_DEVICECHANGE, WM_SETTINGCHANGE,
  },
};

use crate::{platform_impl::Installable, DisplayEvent};

thread_local! {
  static DISPLAY_EVENT_TX: OnceLock<tokio::sync::mpsc::UnboundedSender<crate::DisplayEvent>> = const { OnceLock::new() };
}

pub struct DisplayHook {
  event_rx: tokio::sync::mpsc::UnboundedReceiver<crate::DisplayEvent>,
}

impl DisplayHook {
  // Can't simplfy this since closures don't have a dedicated type.
  #[allow(clippy::type_complexity)]
  pub fn new() -> anyhow::Result<(
    Self,
    Installable<
      impl FnOnce() -> anyhow::Result<()>,
      impl FnOnce() -> anyhow::Result<()>,
    >,
  )> {
    let (event_tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let install = move || -> anyhow::Result<()> {
      DISPLAY_EVENT_TX.with(|tx| {
        tx.set(event_tx).map_err(|_| {
          anyhow::anyhow!("Failed to set global display event sender")
        })
      })
    };

    let stop = || {
      tracing::info!("Stopping display hook");
      Ok(())
    };

    let install = Installable {
      installer: install,
      stop,
    };

    Ok((Self { event_rx: rx }, install))
  }

  pub async fn next_event(&mut self) -> Option<crate::DisplayEvent> {
    self.event_rx.recv().await
  }

  pub fn try_next_event(&mut self) -> Option<crate::DisplayEvent> {
    self.event_rx.try_recv().ok()
  }

  /// Handles raw input messages for mouse events and emits the
  /// corresponding platform event through an MPSC channel.
  pub fn handle_display_event(
    msg: u32,
    wparam: WPARAM,
  ) -> anyhow::Result<()> {
    let event_tx = DISPLAY_EVENT_TX
      .with(|tx| tx.get().cloned())
      .ok_or(anyhow::anyhow!("No display event sender"))?;
    #[allow(clippy::cast_possible_truncation)]
    let should_emit_event = match msg {
      WM_SETTINGCHANGE => {
        wparam.0 as u32 == SPI_SETWORKAREA.0
          || wparam.0 as u32 == SPI_ICONVERTICALSPACING.0
      }
      WM_DEVICECHANGE => wparam.0 as u32 == DBT_DEVNODES_CHANGED,
      _ => true,
    };

    if should_emit_event {
      event_tx.send(DisplayEvent::DisplaySettingsChanged)?;
    }

    Ok(())
  }
}
