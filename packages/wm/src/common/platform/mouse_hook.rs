use std::sync::{
  atomic::{AtomicBool, AtomicU32, Ordering},
  Arc, Mutex, OnceLock,
};

use tokio::sync::mpsc;
use tracing::{error, warn};
use windows::Win32::{
  Foundation::{LPARAM, LRESULT, WPARAM},
  UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
    MSLLHOOKSTRUCT, WH_MOUSE_LL, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEMOVE, WM_RBUTTONDOWN, WM_RBUTTONUP,
  },
};

use crate::common::Point;

use super::{MouseMoveEvent, PlatformEvent};

/// Global instance of `MouseHook`.
///
/// For use with hook procedure.
static MOUSE_HOOK: OnceLock<Arc<MouseHook>> = OnceLock::new();

#[derive(Debug)]
pub struct MouseHook {
  /// Sender to emit platform events.
  event_tx: mpsc::UnboundedSender<PlatformEvent>,

  /// Handle to the mouse hook.
  hook: Arc<Mutex<HHOOK>>,

  /// Whether left-click is currently pressed.
  is_l_mouse_down: AtomicBool,

  /// Whether right-click is currently pressed.
  is_r_mouse_down: AtomicBool,

  /// Timestamp of the last event emission.
  last_event_time: AtomicU32,
}

impl MouseHook {
  /// Creates an instance of `MouseHook`.
  pub fn new(
    event_tx: mpsc::UnboundedSender<PlatformEvent>,
  ) -> anyhow::Result<Arc<Self>> {
    let mouse_hook = Arc::new(Self {
      event_tx,
      hook: Arc::new(Mutex::new(HHOOK::default())),
      is_l_mouse_down: AtomicBool::new(false),
      is_r_mouse_down: AtomicBool::new(false),
      last_event_time: AtomicU32::new(0),
    });

    MOUSE_HOOK
      .set(mouse_hook.clone())
      .map_err(|_| anyhow::anyhow!("Mouse hook already running."))?;

    Ok(mouse_hook)
  }

  /// Starts a mouse hook on the current thread.
  ///
  /// Assumes that a message loop is currently running.
  pub fn start(&self) -> anyhow::Result<()> {
    *self.hook.lock().unwrap() = unsafe {
      SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), None, 0)
    }?;

    Ok(())
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
  pub fn stop(&self) {
    if let Err(err) =
      unsafe { UnhookWindowsHookEx(*self.hook.lock().unwrap()) }
    {
      error!("Failed to unhook low-level mouse hook: {}", err);
    }
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

    if let Some(hook) = MOUSE_HOOK.get() {
      hook.handle_event(wparam.0 as u32, mouse_event);
    }
  }

  unsafe { CallNextHookEx(None, code, wparam, lparam) }
}
