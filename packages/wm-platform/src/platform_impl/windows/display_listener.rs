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
              // System is resuming from sleep/hibernation.
              PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND => {
                is_system_suspended.store(false, Ordering::Relaxed);
              }
              // System is entering sleep/hibernation.
              PBT_APMSUSPEND => {
                is_system_suspended.store(true, Ordering::Relaxed);
              }
              _ => {}
            }

            Some(LRESULT(0))
          }
          WM_DISPLAYCHANGE | WM_SETTINGCHANGE | WM_DEVICECHANGE => {
            let should_emit = {
              // Ignore display change messages if the system hasn't fully
              // resumed from sleep.
              if is_system_suspended.load(Ordering::Relaxed) {
                false
              } else {
                #[allow(clippy::cast_possible_truncation)]
                match message {
                  // Received when displays are connected and disconnected,
                  // resolution changes, or arrangement changes.
                  WM_DISPLAYCHANGE => true,
                  // Received when the working area has changed. Fires when
                  // the Windows taskbar is changed or an appbar is
                  // registered or changed. 3rd-party apps like
                  // ButteryTaskbar can trigger this message by calling
                  // `SystemParametersInfo(SPI_SETWORKAREA, ...)`.
                  WM_SETTINGCHANGE => wparam.0 as u32 == SPI_SETWORKAREA.0,
                  // Received when any device is connected or disconnected
                  // (including non-display devices).
                  // TODO: Check if this is actually needed. Previous C#
                  // implementation did not use this.
                  WM_DEVICECHANGE => {
                    wparam.0 as u32 == DBT_DEVNODES_CHANGED
                  }
                }
              }
            };

            if should_emit {
              let _ = event_tx.send(());
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
