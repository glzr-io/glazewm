use std::{
  cell::OnceCell,
  thread::{self, JoinHandle},
};

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use windows::Win32::{
  Foundation::{HWND, LPARAM, LRESULT, WPARAM},
  UI::WindowsAndMessaging::{
    DefWindowProcW, DestroyWindow, PostQuitMessage, DBT_DEVNODES_CHANGED,
    SPI_ICONVERTICALSPACING, SPI_SETWORKAREA, WM_DESTROY, WM_DEVICECHANGE,
    WM_DISPLAYCHANGE, WM_POWERBROADCAST, WM_SETTINGCHANGE,
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
  window_thread: Option<JoinHandle<Result<()>>>,
}

impl EventWindow {
  pub fn new(
    event_tx: mpsc::UnboundedSender<PlatformEvent>,
    options: EventWindowOptions,
  ) -> Self {
    let window_thread = thread::spawn(move || {
      // Initialize the thread-local sender for platform events.
      PLATFORM_EVENT_TX
        .with(|cell| cell.set(event_tx.clone()))
        .map_err(|_| {
          anyhow::anyhow!("Platform event sender already set.")
        })?;

      // Start hooks for listening to platform events.
      KeyboardHook::start(options.keybindings, event_tx.clone())?;
      WinEventHook::start(event_tx.clone())?;
      MouseHook::start(options.enable_mouse_events, event_tx)?;

      // Create a hidden window with a message loop on the current thread.
      let handle =
        Platform::create_message_window(Some(event_window_proc))?;

      Platform::run_message_loop();
      unsafe { DestroyWindow(HWND(handle)) }?;

      Ok(())
    });

    Self {
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
  pub fn destroy(&mut self) -> anyhow::Result<()> {
    info!("Shutting down event window.");

    // Wait for the spawned thread to finish.
    if let Some(window_thread) = self.window_thread.take() {
      Platform::kill_message_loop(&window_thread)?;

      window_thread
        .join()
        .map_err(|_| anyhow::anyhow!("Thread join failed."))??;
    }

    Ok(())
  }
}

impl Drop for EventWindow {
  fn drop(&mut self) {
    if let Err(err) = self.destroy() {
      error!("Failed to gracefully shut down event window: {}", err);
    }
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
        WM_POWERBROADCAST => {
          event_tx.send(PlatformEvent::PowerModeChanged).unwrap();
          LRESULT(0)
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
