use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};

use tokio::sync::mpsc;
use tracing::warn;
use windows::Win32::{
  Foundation::{HWND, LPARAM, LRESULT, WPARAM},
  UI::WindowsAndMessaging::{
    DBT_DEVNODES_CHANGED, PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND,
    PBT_APMSUSPEND, SPI_ICONVERTICALSPACING, SPI_SETWORKAREA,
    WM_DEVICECHANGE, WM_DISPLAYCHANGE, WM_POWERBROADCAST,
    WM_SETTINGCHANGE,
  },
};

use crate::{Dispatcher, DispatcherExtWindows};

/// Listens for display changes via the event loop's message window.
pub struct DisplayListener {
  event_rx: mpsc::UnboundedReceiver<()>,
  callback_id: Option<usize>,
  dispatcher: Dispatcher,
}

impl DisplayListener {
  /// Creates a new `DisplayListener` instance.
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let is_system_suspended = Arc::new(AtomicBool::new(false));

    let callback_id = dispatcher.register_wndproc_callback(Box::new(
      move |_hwnd: HWND,
            message: u32,
            wparam: WPARAM,
            _lparam: LPARAM|
            -> Option<LRESULT> {
        match message {
          WM_POWERBROADCAST => {
            #[allow(clippy::cast_possible_truncation)]
            match wparam.0 as u32 {
              PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND => {
                is_system_suspended.store(false, Ordering::Relaxed);
              }
              PBT_APMSUSPEND => {
                is_system_suspended.store(true, Ordering::Relaxed);
              }
              _ => {}
            }

            Some(LRESULT(0))
          }
          WM_DISPLAYCHANGE | WM_SETTINGCHANGE | WM_DEVICECHANGE => {
            if !is_system_suspended.load(Ordering::Relaxed) {
              #[allow(clippy::cast_possible_truncation)]
              let should_emit = match message {
                WM_SETTINGCHANGE => {
                  wparam.0 as u32 == SPI_SETWORKAREA.0
                    || wparam.0 as u32 == SPI_ICONVERTICALSPACING.0
                }
                WM_DEVICECHANGE => wparam.0 as u32 == DBT_DEVNODES_CHANGED,
                _ => true,
              };

              if should_emit {
                let _ = event_tx.send(());
              }
            }

            Some(LRESULT(0))
          }
          _ => None,
        }
      },
    ))?;

    Ok(Self {
      event_rx,
      callback_id: Some(callback_id),
      dispatcher: dispatcher.clone(),
    })
  }

  /// Returns when the next display settings change is detected.
  ///
  /// Returns `None` if the channel has been closed.
  pub async fn next_event(&mut self) -> Option<()> {
    self.event_rx.recv().await
  }

  /// Deregisters the window procedure callback.
  pub fn terminate(&mut self) -> crate::Result<()> {
    if let Some(id) = self.callback_id.take() {
      self.dispatcher.deregister_wndproc_callback(id)?;
    }

    Ok(())
  }
}

impl Drop for DisplayListener {
  fn drop(&mut self) {
    if let Err(err) = self.terminate() {
      warn!("Failed to terminate display listener: {}", err);
    }
  }
}
