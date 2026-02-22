use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};

use tokio::sync::mpsc;
use tracing::warn;
use windows::Win32::UI::WindowsAndMessaging::{
  DBT_DEVNODES_CHANGED, PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND,
  PBT_APMSUSPEND, SPI_SETWORKAREA, WM_DEVICECHANGE, WM_DISPLAYCHANGE,
  WM_POWERBROADCAST, WM_SETTINGCHANGE,
};

use crate::{Dispatcher, DispatcherExtWindows};

/// Windows-specific implementation of [`DisplayListener`].
pub(crate) struct DisplayListener {
  callback_id: Option<usize>,
  dispatcher: Dispatcher,
}

impl DisplayListener {
  /// Windows-specific implementation of [`DisplayListener::new`].
  pub(crate) fn new(
    event_tx: mpsc::UnboundedSender<()>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let is_system_suspended = Arc::new(AtomicBool::new(false));

    let callback_id = dispatcher.register_wndproc_callback(Box::new(
      move |_hwnd, message, wparam, _lparam| {
        match message {
          WM_POWERBROADCAST => {
            #[allow(clippy::cast_possible_truncation)]
            match wparam as u32 {
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

            Some(0)
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
                  WM_SETTINGCHANGE => wparam as u32 == SPI_SETWORKAREA.0,
                  // Received when any device is connected or disconnected
                  // (including non-display devices).
                  // TODO: Check if this is actually needed. Previous C#
                  // implementation did not use this.
                  WM_DEVICECHANGE => {
                    wparam as u32 == DBT_DEVNODES_CHANGED
                  }
                  _ => unreachable!(),
                }
              }
            };

            if should_emit {
              let _ = event_tx.send(());
            }

            Some(0)
          }
          _ => None,
        }
      },
    ))?;

    Ok(Self {
      callback_id: Some(callback_id),
      dispatcher: dispatcher.clone(),
    })
  }

  /// Windows-specific implementation of [`DisplayListener::terminate`].
  pub(crate) fn terminate(&mut self) -> crate::Result<()> {
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
