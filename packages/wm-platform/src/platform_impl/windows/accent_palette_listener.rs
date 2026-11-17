use std::cell::RefCell;

use tokio::sync::mpsc;
use tracing::warn;
use windows::{
  core::w,
  Win32::{
    System::Registry::{
      RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_BINARY,
    },
    UI::WindowsAndMessaging::WM_DWMCOLORIZATIONCOLORCHANGED,
  },
};

use crate::{AccentPalette, Dispatcher, DispatcherExtWindows};

/// Platform-specific implementation of [`AccentPaletteListener`].
pub(crate) struct AccentPaletteListener {
  callback_id: Option<usize>,
  dispatcher: Dispatcher,
}

impl AccentPaletteListener {
  /// Implements [`AccentPaletteListener::new`].
  pub(crate) fn new(
    event_tx: mpsc::UnboundedSender<AccentPalette>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let last_palette = RefCell::new(accent_palette().ok());

    let callback_id = dispatcher.register_wndproc_callback(Box::new(
      move |_hwnd, message, _wparam, _lparam| {
        // Broadcast to top-level windows on accent changes, after the
        // `AccentPalette` registry value has been updated. Sent several
        // times per change, hence the deduplication.
        if message == WM_DWMCOLORIZATIONCOLORCHANGED {
          match accent_palette() {
            Ok(palette) => {
              if last_palette.borrow().as_ref() != Some(&palette) {
                last_palette.replace(Some(palette.clone()));
                let _ = event_tx.send(palette);
              }
            }
            Err(err) => warn!("Failed to read accent palette: {err}"),
          }
        }

        // Observe only, such that other callbacks also receive the
        // broadcast.
        None
      },
    ))?;

    Ok(Self {
      callback_id: Some(callback_id),
      dispatcher: dispatcher.clone(),
    })
  }

  /// Implements [`AccentPaletteListener::terminate`].
  pub(crate) fn terminate(&mut self) -> crate::Result<()> {
    if let Some(id) = self.callback_id.take() {
      self.dispatcher.deregister_wndproc_callback(id)?;
    }

    Ok(())
  }
}

impl Drop for AccentPaletteListener {
  fn drop(&mut self) {
    if let Err(err) = self.terminate() {
      warn!("Failed to terminate accent palette listener: {}", err);
    }
  }
}

/// Reads the system accent palette from the registry.
pub(crate) fn accent_palette() -> crate::Result<AccentPalette> {
  let mut bytes = [0u8; 32];
  #[allow(clippy::cast_possible_truncation)]
  let mut size = bytes.len() as u32;

  unsafe {
    RegGetValueW(
      HKEY_CURRENT_USER,
      w!(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Accent"),
      w!("AccentPalette"),
      RRF_RT_REG_BINARY,
      None,
      Some(bytes.as_mut_ptr().cast()),
      Some(&raw mut size),
    )
  }?;

  AccentPalette::from_registry_bytes(&bytes[..size as usize])
}
