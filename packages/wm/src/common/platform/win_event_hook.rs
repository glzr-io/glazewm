use std::{cell::OnceCell, sync::Arc};

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::warn;
use windows::Win32::{
  Foundation::HWND,
  UI::{
    Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
    WindowsAndMessaging::{
      EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE,
      EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE,
      EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED, EVENT_SYSTEM_FOREGROUND,
      EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART,
      EVENT_SYSTEM_MOVESIZEEND, OBJID_WINDOW, WINEVENT_OUTOFCONTEXT,
      WINEVENT_SKIPOWNPROCESS,
    },
  },
};

use super::{NativeWindow, PlatformEvent};

thread_local! {
  /// Thread-local for instance of `WinEventHook`.
  ///
  /// For use with hook procedure.
  static WINDOW_EVENT_HOOK: OnceCell<Arc<WinEventHook>> = OnceCell::new();
}

pub struct WinEventHook {
  event_tx: mpsc::UnboundedSender<PlatformEvent>,
  hook_handles: Vec<HWINEVENTHOOK>,
}

impl WinEventHook {
  /// Starts a window event hook on the current thread.
  ///
  /// Assumes that a message loop is currently running.
  pub fn start(
    event_tx: mpsc::UnboundedSender<PlatformEvent>,
  ) -> Result<Arc<Self>> {
    let hook_handles = Self::hook_win_events()?;

    let window_event_hook = Arc::new(Self {
      event_tx,
      hook_handles,
    });

    WINDOW_EVENT_HOOK
      .with(|cell| cell.set(window_event_hook.clone()))
      .map_err(|_| {
        anyhow::anyhow!(
          "Window event hook already started on current thread."
        )
      })?;

    Ok(window_event_hook)
  }

  fn handle_event(&self, event_type: u32, handle: isize) {
    let window = NativeWindow::new(handle);

    let platform_event = match event_type {
      EVENT_OBJECT_DESTROY => PlatformEvent::WindowDestroyed(window),
      EVENT_SYSTEM_FOREGROUND => PlatformEvent::WindowFocused(window),
      EVENT_OBJECT_HIDE | EVENT_OBJECT_CLOAKED => {
        PlatformEvent::WindowHidden(window)
      }
      EVENT_OBJECT_LOCATIONCHANGE => {
        PlatformEvent::WindowLocationChanged(window)
      }
      EVENT_SYSTEM_MINIMIZESTART => PlatformEvent::WindowMinimized(window),
      EVENT_SYSTEM_MINIMIZEEND => {
        PlatformEvent::WindowMinimizeEnded(window)
      }
      EVENT_SYSTEM_MOVESIZEEND => {
        PlatformEvent::WindowMovedOrResized(window)
      }
      EVENT_OBJECT_SHOW | EVENT_OBJECT_UNCLOAKED => {
        PlatformEvent::WindowShown(window)
      }
      EVENT_OBJECT_NAMECHANGE => PlatformEvent::WindowTitleChanged(window),
      _ => return,
    };

    if let Err(err) = self.event_tx.send(platform_event) {
      warn!("Failed to send platform event '{}'.", err);
    }
  }

  /// Creates several window event hooks via `SetWinEventHook`.
  fn hook_win_events() -> Result<Vec<HWINEVENTHOOK>> {
    let event_ranges = [
      (EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_LOCATIONCHANGE),
      (EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE),
      (EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MINIMIZEEND),
      (EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZEEND),
      (EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND),
      (EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE),
      (EVENT_OBJECT_CLOAKED, EVENT_OBJECT_UNCLOAKED),
    ];

    // Create separate hooks for each event range. This is more performant
    // than creating a single hook for all events and filtering them.
    event_ranges
      .iter()
      .try_fold(Vec::new(), |mut handles, event_range| {
        let hook_handle =
          Self::hook_win_event(event_range.0, event_range.1)?;
        handles.push(hook_handle);
        Ok(handles)
      })
  }

  /// Creates a window hook for the specified event range.
  fn hook_win_event(
    event_min: u32,
    event_max: u32,
  ) -> Result<HWINEVENTHOOK> {
    let hook_handle = unsafe {
      SetWinEventHook(
        event_min,
        event_max,
        None,
        Some(window_event_hook_proc),
        0,
        0,
        WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
      )
    };

    if hook_handle.is_invalid() {
      Err(anyhow::anyhow!("Failed to set window event hook."))
    } else {
      Ok(hook_handle)
    }
  }

  /// Stops the window event hook and unhooks from all window events.
  pub fn stop(&mut self) {
    for hook_handle in self.hook_handles.drain(..) {
      if let false = unsafe { UnhookWinEvent(hook_handle) }.as_bool() {
        warn!("Failed to unhook window event.");
      }
    }
  }
}

impl Drop for WinEventHook {
  fn drop(&mut self) {
    self.stop();
  }
}

/// Callback passed to `SetWinEventHook` to handle window events.
///
/// This function is called on selected window events, and forwards them
/// through an MPSC channel for the WM to process.
extern "system" fn window_event_hook_proc(
  _hook: HWINEVENTHOOK,
  event_type: u32,
  handle: HWND,
  id_object: i32,
  id_child: i32,
  _event_thread: u32,
  _event_time: u32,
) {
  let is_window_event =
    id_object == OBJID_WINDOW.0 && id_child == 0 && handle != HWND(0);

  // Check whether the event is associated with a window object instead
  // of a UI control.
  if !is_window_event {
    return;
  }

  WINDOW_EVENT_HOOK.with(|hook| {
    if let Some(hook) = hook.get() {
      hook.handle_event(event_type, handle.0);
    }
  });
}
