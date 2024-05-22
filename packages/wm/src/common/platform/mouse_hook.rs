use std::{
  cell::OnceCell,
  sync::{
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc,
  },
};

use crate::common::Point;

use super::{MouseMoveEvent, PlatformEvent};
use anyhow::Result;
use tokio::sync::mpsc;
use tracing::warn;
use windows::Win32::{
  Foundation::{LPARAM, LRESULT, WPARAM},
  UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
    MSLLHOOKSTRUCT, WH_MOUSE_LL, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEMOVE, WM_RBUTTONDOWN, WM_RBUTTONUP,
  },
};

thread_local! {
  /// Thread-local for instance of `MouseHook`.
  ///
  /// For use with hook procedure.
  static MOUSE_HOOK: OnceCell<Arc<MouseHook>> = OnceCell::new();
}

pub struct MouseHook {
  /// Sender to emit platform events.
  event_tx: mpsc::UnboundedSender<PlatformEvent>,

  /// Handle to the mouse hook.
  hook: Option<HHOOK>,

  /// Whether left-click is currently pressed.
  is_l_mouse_down: AtomicBool,

  /// Whether right-click is currently pressed.
  is_r_mouse_down: AtomicBool,

  /// Timestamp of the last event emission.
  last_event_time: AtomicU32,
}

impl MouseHook {
  /// Starts a mouse hook on the current thread.
  ///
  /// Assumes that a message loop is currently running.
  pub fn start(
    enabled: bool,
    event_tx: mpsc::UnboundedSender<PlatformEvent>,
  ) -> Result<Arc<MouseHook>> {
    let hook = match enabled {
      false => None,
      true => {
        let hook = unsafe {
          SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0)
        }?;

        Some(hook)
      }
    };

    let mouse_hook = Arc::new(Self {
      event_tx,
      hook,
      is_l_mouse_down: AtomicBool::new(false),
      is_r_mouse_down: AtomicBool::new(false),
      last_event_time: AtomicU32::new(0),
    });

    MOUSE_HOOK
      .with(|cell| cell.set(mouse_hook.clone()))
      .map_err(|_| {
        anyhow::anyhow!("Mouse hook already started on current thread.")
      })?;

    Ok(mouse_hook)
  }

  fn handle_event(&self, event_type: u32, mouse_event: MSLLHOOKSTRUCT) {
    // Throttle events so that there's a minimum of 50ms between each
    // emission.
    let last_event_time = self.last_event_time.load(Ordering::Relaxed);
    if mouse_event.time - last_event_time < 50 {
      return;
    }

    match event_type {
      WM_LBUTTONDOWN => {
        self.is_l_mouse_down.store(true, Ordering::Relaxed)
      }
      WM_LBUTTONUP => self.is_l_mouse_down.store(false, Ordering::Relaxed),
      WM_RBUTTONDOWN => {
        self.is_r_mouse_down.store(true, Ordering::Relaxed)
      }
      WM_RBUTTONUP => self.is_r_mouse_down.store(false, Ordering::Relaxed),
      WM_MOUSEMOVE => {
        let is_mouse_down = self.is_l_mouse_down.load(Ordering::Relaxed)
          || self.is_r_mouse_down.load(Ordering::Relaxed);

        let event = MouseMoveEvent {
          point: Point {
            x: mouse_event.pt.x,
            y: mouse_event.pt.y,
          },
          is_mouse_down,
        };

        if let Err(err) =
          self.event_tx.send(PlatformEvent::MouseMove(event))
        {
          warn!("Failed to send platform event '{}'.", err);
        }

        self
          .last_event_time
          .store(mouse_event.time, Ordering::Relaxed);
      }
      _ => {}
    }
  }

  /// Stops the low-level mouse hook.
  pub fn stop(&mut self) -> anyhow::Result<()> {
    if let Some(hook) = self.hook.take() {
      unsafe { UnhookWindowsHookEx(hook) }?;
    }

    Ok(())
  }
}

impl Drop for MouseHook {
  fn drop(&mut self) {
    let _ = self.stop();
  }
}

/// Callback function for the low-level mouse hook.
///
/// This function is called whenever a mouse event occurs, and it forwards
/// the event through an MPSC channel for the WM to process.
extern "system" fn mouse_hook_proc(
  code: i32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  if code >= 0 {
    let mouse_event = unsafe { *(lparam.0 as *const MSLLHOOKSTRUCT) };

    MOUSE_HOOK.with(|hook| {
      if let Some(hook) = hook.get() {
        hook.handle_event(wparam.0 as u32, mouse_event);
      }
    });
  }

  unsafe { CallNextHookEx(None, code, wparam, lparam) }
}
