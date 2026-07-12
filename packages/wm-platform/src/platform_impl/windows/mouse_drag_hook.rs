use std::{
  cell::Cell,
  collections::HashSet,
  sync::{
    atomic::{AtomicBool, AtomicU8, Ordering},
    Arc, Condvar, Mutex, RwLock,
  },
};

use tokio::sync::mpsc;
use windows::Win32::{
  Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
  UI::{
    Input::KeyboardAndMouse::{GetKeyState, VK_LWIN, VK_MENU, VK_RWIN},
    WindowsAndMessaging::{
      CallNextHookEx, GetAncestor, GetWindowRect, SetWindowPos,
      SetWindowsHookExW, UnhookWindowsHookEx, WindowFromPoint, GA_ROOT,
      HHOOK, MSLLHOOKSTRUCT, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER,
      WH_MOUSE_LL, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
    },
  },
};

use crate::{Dispatcher, DragModifier, MouseDragEvent, WindowId};

thread_local! {
  /// Stores the shared state for the hook procedure on the current
  /// thread (i.e. the dispatcher's thread).
  static HOOK: Cell<Option<Arc<SharedState>>> = Cell::default();
}

/// An in-progress grab-and-move drag.
#[derive(Debug)]
struct DragSession {
  /// Handle of the window being dragged.
  hwnd: isize,

  /// Offset from the cursor to the window's top-left corner at grab
  /// time, so the window doesn't jump to the cursor.
  offset_x: i32,
  offset_y: i32,
}

/// The newest position for the mover thread to apply.
#[derive(Clone, Copy, Debug)]
struct MoveTarget {
  hwnd: isize,
  x: i32,
  y: i32,
}

/// Coalescing handoff between the hook (producer) and the mover thread
/// (consumer). The hook always overwrites with the newest position, so
/// a slow window receives one current move whenever it's ready instead
/// of a backlog of stale ones (which renders as a rubber-band slide —
/// observed on Firefox 2026-07-12).
#[derive(Debug, Default)]
struct MoverState {
  target: Mutex<(Option<MoveTarget>, bool)>,
  signal: Condvar,
}

/// State shared between the hook procedure (dispatcher thread) and the
/// WM (main thread).
#[derive(Debug)]
struct SharedState {
  /// Whether grab-and-move is currently active.
  enabled: AtomicBool,

  /// Modifier key that activates a drag (0 = alt, 1 = win).
  modifier: AtomicU8,

  /// Whether a drag session is in progress. Fast-path gate so the
  /// high-volume mouse-move events cost one atomic load when idle.
  dragging: AtomicBool,

  /// The in-progress drag session, if any.
  session: Mutex<Option<DragSession>>,

  /// IDs of windows currently managed by the WM. Only managed windows
  /// can be dragged; ignored/unmanaged windows (e.g. windows matching
  /// `ignore` rules) pass clicks through untouched.
  managed_handles: RwLock<HashSet<WindowId>>,

  /// Sender for drag lifecycle events consumed by the WM.
  event_tx: mpsc::UnboundedSender<MouseDragEvent>,

  /// Handoff to the mover thread.
  mover: MoverState,
}

/// A system-wide low-level mouse hook for grab-and-move: holding a
/// modifier key and left-clicking anywhere on a managed window starts a
/// drag that moves the window with the cursor until the button is
/// released.
///
/// The window is moved manually via `SetWindowPos` on every mouse-move.
/// Delegating to the native modal move loop (posting `WM_NCLBUTTONDOWN`/
/// `HTCAPTION` or `WM_SYSCOMMAND`/`SC_MOVE|2`) was tried first and is
/// ignored by custom-frame applications (Firefox, WezTerm — observed
/// 2026-07-12), which is exactly the class of title-bar-less window this
/// feature exists for. Manual moving works for every window.
///
/// Drag lifecycle events are sent to the WM, which synthesizes the same
/// interactive move start/end handling a native title-bar drag produces
/// — so `active_drag` and tiling reflow behave identically.
///
/// Platform-specific: this is a Windows-only feature (`WH_MOUSE_LL`).
#[derive(Debug)]
pub struct MouseDragHook {
  handle: HHOOK,
  shared: Arc<SharedState>,
  dispatcher: Dispatcher,
}

impl MouseDragHook {
  /// Creates an instance of `MouseDragHook`.
  ///
  /// # Panics
  ///
  /// Panics when attempting to register multiple mouse drag hooks on the
  /// dispatcher's thread.
  pub fn new(
    modifier: &DragModifier,
    enabled: bool,
    event_tx: mpsc::UnboundedSender<MouseDragEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let shared = Arc::new(SharedState {
      enabled: AtomicBool::new(enabled),
      modifier: AtomicU8::new(Self::modifier_id(modifier)),
      dragging: AtomicBool::new(false),
      session: Mutex::new(None),
      managed_handles: RwLock::new(HashSet::new()),
      event_tx,
      mover: MoverState::default(),
    });

    // Dedicated mover thread: applies the newest drag position with a
    // synchronous `SetWindowPos`. Blocking on a slow window here is
    // harmless (the hook thread never waits on it), and coalescing
    // means at most one move is ever in flight.
    let mover_shared = Arc::clone(&shared);
    std::thread::spawn(move || {
      Self::mover_thread(&mover_shared);
    });

    let shared_clone = Arc::clone(&shared);

    let handle = dispatcher.dispatch_sync(move || {
      HOOK.with(|state| {
        assert!(
          state.take().is_none(),
          "Only one mouse drag hook can be registered on the dispatcher's thread."
        );

        state.set(Some(shared_clone));
      });

      unsafe {
        SetWindowsHookExW(
          WH_MOUSE_LL,
          Some(Self::hook_proc),
          HINSTANCE::default(),
          0,
        )
      }
    })??;

    Ok(Self {
      handle,
      shared,
      dispatcher: dispatcher.clone(),
    })
  }

  /// Enables or disables drag interception without unregistering the
  /// hook. Disabling mid-drag ends the active session.
  pub fn set_enabled(&self, enabled: bool) {
    self.shared.enabled.store(enabled, Ordering::Relaxed);

    if !enabled {
      Self::end_session(&self.shared);
    }
  }

  /// Updates the modifier key that activates a drag.
  pub fn set_modifier(&self, modifier: &DragModifier) {
    self
      .shared
      .modifier
      .store(Self::modifier_id(modifier), Ordering::Relaxed);
  }

  /// Replaces the set of managed window IDs. Skips the write lock when
  /// the set is unchanged.
  pub fn update_managed_windows(&self, handles: &[WindowId]) {
    {
      let current = self
        .shared
        .managed_handles
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

      if current.len() == handles.len()
        && handles.iter().all(|handle| current.contains(handle))
      {
        return;
      }
    }

    let mut current = self
      .shared
      .managed_handles
      .write()
      .unwrap_or_else(std::sync::PoisonError::into_inner);

    current.clear();
    current.extend(handles.iter().copied());
  }

  /// Terminates the mouse drag hook by unregistering it.
  pub fn terminate(&mut self) -> crate::Result<()> {
    // Shut down the mover thread.
    {
      let mut pending = self
        .shared
        .mover
        .target
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
      pending.1 = true;
      self.shared.mover.signal.notify_one();
    }

    unsafe { UnhookWindowsHookEx(self.handle) }?;

    // Dispatch cleanup to the event loop thread since the shared state
    // is stored in a thread-local on that thread.
    let _ = self.dispatcher.dispatch_async(|| {
      HOOK.with(|state| {
        state.take();
      });
    });

    Ok(())
  }

  /// Maps a [`DragModifier`] to its stored atomic representation.
  fn modifier_id(modifier: &DragModifier) -> u8 {
    match modifier {
      DragModifier::Alt => 0,
      DragModifier::Win => 1,
    }
  }

  /// Gets whether the specified key is currently pressed.
  ///
  /// Uses `GetKeyState` with the same bit check as the keyboard hook's
  /// `KeyEvent::is_key_down` — the mechanism GlazeWM's own keybinding
  /// matching relies on from this same thread.
  fn is_key_down(key: u16) -> bool {
    unsafe { (GetKeyState(key.into()) & 0x80) == 0x80 }
  }

  /// Ends the active drag session (if any), notifying the WM.
  fn end_session(shared: &SharedState) {
    let session = shared
      .session
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner)
      .take();

    shared.dragging.store(false, Ordering::Relaxed);

    // Drop any stale queued move so it can't land after the drop.
    shared
      .mover
      .target
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner)
      .0 = None;

    if let Some(session) = session {
      let _ = shared.event_tx.send(MouseDragEvent::Ended {
        window_id: WindowId(session.hwnd),
      });
    }
  }

  /// Handles a left-button-down: starts a drag session when the
  /// modifier is held over a managed window. Returns `true` to swallow
  /// the click (the application must never see it).
  ///
  /// Must stay fast: low-level hooks have a system-enforced timeout
  /// (`LowLevelHooksTimeout`) after which Windows silently removes them.
  fn handle_button_down(shared: &SharedState, lparam: LPARAM) -> bool {
    if !shared.enabled.load(Ordering::Relaxed) {
      return false;
    }

    let modifier_down = match shared.modifier.load(Ordering::Relaxed) {
      1 => Self::is_key_down(VK_LWIN.0) || Self::is_key_down(VK_RWIN.0),
      // `VK_MENU` covers both left and right alt.
      _ => Self::is_key_down(VK_MENU.0),
    };

    if !modifier_down {
      return false;
    }

    // SAFETY: For `WH_MOUSE_LL`, `lparam` points to a `MSLLHOOKSTRUCT`.
    let input = unsafe { *(lparam.0 as *const MSLLHOOKSTRUCT) };

    let hwnd = unsafe { WindowFromPoint(input.pt) };
    if hwnd.0 == 0 {
      return false;
    }

    let root = unsafe { GetAncestor(hwnd, GA_ROOT) };
    if root.0 == 0 {
      return false;
    }

    let is_managed = shared
      .managed_handles
      .read()
      .unwrap_or_else(std::sync::PoisonError::into_inner)
      .contains(&WindowId(root.0));

    if !is_managed {
      return false;
    }

    let mut rect = RECT::default();
    if unsafe { GetWindowRect(root, &raw mut rect) }.is_err() {
      return false;
    }

    *shared
      .session
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner) =
      Some(DragSession {
        hwnd: root.0,
        offset_x: input.pt.x - rect.left,
        offset_y: input.pt.y - rect.top,
      });

    shared.dragging.store(true, Ordering::Relaxed);

    // Notify before the first `SetWindowPos`, so the WM's `active_drag`
    // is in place when location-change events start arriving.
    let _ = shared.event_tx.send(MouseDragEvent::Started {
      window_id: WindowId(root.0),
    });

    true
  }

  /// Handles a mouse-move during a drag session: hands the newest
  /// position to the mover thread. Never swallows the move, and does
  /// no syscalls on the hook thread.
  fn handle_mouse_move(shared: &SharedState, lparam: LPARAM) {
    // SAFETY: For `WH_MOUSE_LL`, `lparam` points to a `MSLLHOOKSTRUCT`.
    let input = unsafe { *(lparam.0 as *const MSLLHOOKSTRUCT) };

    let target = {
      let session = shared
        .session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

      session.as_ref().map(|session| MoveTarget {
        hwnd: session.hwnd,
        x: input.pt.x - session.offset_x,
        y: input.pt.y - session.offset_y,
      })
    };

    if let Some(target) = target {
      let mut pending = shared
        .mover
        .target
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

      // Overwrite — only the newest position matters.
      pending.0 = Some(target);
      shared.mover.signal.notify_one();
    }
  }

  /// Body of the mover thread: waits for a target position and applies
  /// it with a synchronous `SetWindowPos`, always taking the newest
  /// pending position.
  fn mover_thread(shared: &SharedState) {
    loop {
      let target = {
        let mut pending = shared
          .mover
          .target
          .lock()
          .unwrap_or_else(std::sync::PoisonError::into_inner);

        while pending.0.is_none() && !pending.1 {
          pending = shared
            .mover
            .signal
            .wait(pending)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        }

        if pending.1 {
          return;
        }

        pending.0.take()
      };

      if let Some(target) = target {
        let _ = unsafe {
          SetWindowPos(
            HWND(target.hwnd),
            HWND::default(),
            target.x,
            target.y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
          )
        };
      }
    }
  }

  /// Hook procedure for mouse events.
  ///
  /// For use with `SetWindowsHookExW`.
  extern "system" fn hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {
    // If the code is less than zero, the hook procedure must pass the
    // hook notification directly to other applications.
    if code != 0 {
      return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    #[allow(clippy::cast_possible_truncation)]
    let message = wparam.0 as u32;

    let should_intercept = HOOK.with(|state| {
      let Some(shared) = state.take() else {
        return false;
      };

      let result = match message {
        WM_LBUTTONDOWN => Self::handle_button_down(&shared, lparam),
        WM_MOUSEMOVE => {
          // Fast path: one atomic load when no drag is in progress.
          if shared.dragging.load(Ordering::Relaxed) {
            Self::handle_mouse_move(&shared, lparam);
          }

          false
        }
        WM_LBUTTONUP => {
          if shared.dragging.load(Ordering::Relaxed) {
            Self::end_session(&shared);

            // Swallow the button-up to pair with the swallowed
            // button-down — the application saw neither.
            true
          } else {
            false
          }
        }
        _ => false,
      };

      state.set(Some(shared));
      result
    });

    if should_intercept {
      return LRESULT(1);
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
  }
}

impl Drop for MouseDragHook {
  fn drop(&mut self) {
    let _ = self.terminate();
  }
}
