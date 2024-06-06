use std::{
  sync::{Arc, Mutex, OnceLock},
  thread::{self, JoinHandle},
};

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

/// Global instance of sender for platform events.
///
/// For use with window procedure.
static PLATFORM_EVENT_TX: OnceLock<mpsc::UnboundedSender<PlatformEvent>> =
  OnceLock::new();

#[derive(Debug)]
pub struct EventWindow {
  window_thread: Option<JoinHandle<anyhow::Result<()>>>,
  keyboard_hook: Arc<KeyboardHook>,
}

impl EventWindow {
  /// Creates an instance of `EventWindow`. Spawns a hidden window and
  /// emits platform events.
  ///
  /// Uses global state (i.e. `PLATFORM_EVENT_TX`) and should thus only
  /// ever be instantiated once in the application's lifetime.
  pub fn new(
    event_tx: mpsc::UnboundedSender<PlatformEvent>,
    keybindings: Vec<KeybindingConfig>,
    enable_mouse_events: bool,
  ) -> anyhow::Result<Self> {
    let keyboard_hook = KeyboardHook::new(keybindings, event_tx.clone())?;

    let keyboard_hook_clone = keyboard_hook.clone();

    let window_thread = thread::spawn(move || {
      // Add the sender for platform events to global state.
      PLATFORM_EVENT_TX.set(event_tx.clone()).map_err(|_| {
        anyhow::anyhow!("Platform event sender already set.")
      })?;

      // Start hooks for listening to platform events.
      keyboard_hook_clone.start()?;
      WinEventHook::start(event_tx.clone())?;
      MouseHook::start(enable_mouse_events, event_tx)?;

      // Create a hidden window with a message loop on the current thread.
      let handle =
        Platform::create_message_window(Some(event_window_proc))?;

      Platform::run_message_loop();
      unsafe { DestroyWindow(HWND(handle)) }?;

      Ok(())
    });

    Ok(Self {
      keyboard_hook,
      window_thread: Some(window_thread),
    })
  }

  pub fn update_keybindings(
    &mut self,
    keybindings: Vec<KeybindingConfig>,
  ) {
    self.keyboard_hook.update(keybindings);
  }

  pub fn enable_mouse_listener(&mut self, is_enabled: bool) {
    // TODO
  }

  /// Destroys the event window and stops the message loop.
  pub fn destroy(&mut self) -> anyhow::Result<()> {
    info!("Shutting down event window.");

    let _ = self.keyboard_hook.stop();

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
  if let Some(event_tx) = PLATFORM_EVENT_TX.get() {
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
