use std::{
  cell::OnceCell,
  thread::{self, JoinHandle},
};

use anyhow::Result;
use tokio::sync::{mpsc, oneshot};
use tracing::warn;
use windows::Win32::{
  Foundation::{HWND, LPARAM, LRESULT, WPARAM},
  UI::WindowsAndMessaging::{
    DefWindowProcW, PostQuitMessage, DBT_DEVNODES_CHANGED,
    SPI_ICONVERTICALSPACING, SPI_SETWORKAREA, WM_DESTROY, WM_DEVICECHANGE,
    WM_DISPLAYCHANGE, WM_SETTINGCHANGE,
  },
};

use crate::user_config::KeybindingConfig;

use super::{
  KeyboardHook, MouseHook, Platform, PlatformEvent, WinEventHook,
};

thread_local! {
  static PLATFORM_EVENT_TX: OnceCell<mpsc::UnboundedSender<PlatformEvent>> = OnceCell::new();
}

#[derive(Debug)]
pub struct EventWindowOptions {
  pub keybindings: Vec<KeybindingConfig>,
  pub enable_mouse_events: bool,
}

#[derive(Debug)]
pub struct EventWindow {
  abort_tx: Option<oneshot::Sender<()>>,
  window_thread: Option<JoinHandle<Result<()>>>,
}

impl EventWindow {
  pub fn new(
    event_tx: mpsc::UnboundedSender<PlatformEvent>,
    options: EventWindowOptions,
  ) -> Self {
    let (abort_tx, abort_rx) = oneshot::channel();

    let window_thread = thread::spawn(|| unsafe {
      // Initialize the thread-local sender for platform events.
      PLATFORM_EVENT_TX
        .with(|cell| cell.set(event_tx.clone()))
        .map_err(|_| {
          anyhow::anyhow!("Platform event sender already set.")
        })?;

      // Start hooks for listening to platform events.
      KeyboardHook::start(options.keybindings, event_tx.clone())?;
      WinEventHook::start(event_tx.clone())?;
      MouseHook::start(event_tx)?;

      // Create a hidden window with a message loop on the current thread.
      Platform::create_message_loop(abort_rx, Some(event_window_proc))?;
      Ok(())
    });

    Self {
      abort_tx: Some(abort_tx),
      window_thread: Some(window_thread),
    }
  }

  pub fn update_keybindings(
    &mut self,
    keybindings: Vec<KeybindingConfig>,
  ) {
    todo!()
  }

  pub fn enable_mouse_listener(&mut self, is_enabled: bool) {
    todo!()
  }

  /// Destroys the event window and stops the message loop.
  pub fn destroy(&mut self) {
    if let Some(abort_tx) = self.abort_tx.take() {
      if abort_tx.send(()).is_err() {
        warn!("Failed to send abort signal to the event window thread.");
      }
    }

    // Wait for the spawned thread to finish.
    if let Some(window_thread) = self.window_thread.take() {
      if let Err(err) = window_thread.join() {
        warn!("Failed to join event window thread '{:?}'.", err);
      }
    }
  }
}

impl Drop for EventWindow {
  fn drop(&mut self) {
    self.destroy();
  }
}

/// Window procedure for the event window.
///
/// Handles messages for the event window, and forwards display change
/// events through an MPSC channel for the WM to process.
pub extern "system" fn event_window_proc(
  handle: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  PLATFORM_EVENT_TX.with(|event_tx| {
    if let Some(event_tx) = event_tx.get() {
      return match message {
        WM_DISPLAYCHANGE | WM_SETTINGCHANGE | WM_DEVICECHANGE => {
          handle_display_change_msg(message, wparam, event_tx)
        }
        WM_DESTROY => {
          unsafe { PostQuitMessage(0) };
          LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(handle, message, wparam, lparam) },
      };
    }

    LRESULT(0)
  })
}

/// Handles display change messages and emits the corresponding platform
/// event through an MPSC channel.
fn handle_display_change_msg(
  message: u32,
  wparam: WPARAM,
  event_tx: &mpsc::UnboundedSender<PlatformEvent>,
) -> LRESULT {
  let should_emit_event = match message {
    WM_SETTINGCHANGE => {
      wparam.0 as u32 == SPI_SETWORKAREA.0
        || wparam.0 as u32 == SPI_ICONVERTICALSPACING.0
    }
    WM_DEVICECHANGE => wparam.0 as u32 == DBT_DEVNODES_CHANGED,
    _ => true,
  };

  if should_emit_event {
    let event = PlatformEvent::DisplaySettingsChanged;
    if let Err(err) = event_tx.send(event) {
      warn!("Failed to send platform event '{}'.", err);
    }
  }

  LRESULT(0)
}
