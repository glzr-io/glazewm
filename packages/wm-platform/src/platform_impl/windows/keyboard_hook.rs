use std::cell::Cell;

use windows::Win32::{
  Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM},
  UI::{
    Input::KeyboardAndMouse::{
      GetKeyState, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RCONTROL,
      VK_RMENU, VK_RSHIFT, VK_RWIN,
    },
    WindowsAndMessaging::{
      CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
      KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
    },
  },
};

use crate::{Dispatcher, Key, KeyCode};

thread_local! {
  /// Stores the hook callback for the current thread.
  ///
  /// The hook callback is called for every keyboard event and returns
  /// `true` if the event should be intercepted.
  static HOOK: Cell<Option<Box<dyn Fn(KeyEvent) -> bool>>> = Cell::default();
}

/// Windows-specific keyboard event.
#[derive(Clone, Debug)]
pub struct KeyEvent {
  /// The key that was pressed or released.
  pub key: Key,

  /// Key code that generated this event.
  pub key_code: KeyCode,

  /// Whether the event is for a key press or release.
  pub is_keypress: bool,
}

impl KeyEvent {
  /// Creates an instance of `KeyEvent`.
  pub(crate) fn new(
    key: Key,
    key_code: KeyCode,
    is_keypress: bool,
  ) -> Self {
    Self {
      is_keypress,
      key_code,
      key,
    }
  }

  /// Gets whether the specified key is currently pressed.
  pub fn is_key_down(&self, key: Key) -> bool {
    match key {
      Key::Cmd | Key::Win => {
        Self::is_key_down_raw(VK_LWIN.0)
          || Self::is_key_down_raw(VK_RWIN.0)
      }
      Key::Alt => {
        Self::is_key_down_raw(VK_LMENU.0)
          || Self::is_key_down_raw(VK_RMENU.0)
      }
      Key::Ctrl => {
        Self::is_key_down_raw(VK_LCONTROL.0)
          || Self::is_key_down_raw(VK_RCONTROL.0)
      }
      Key::Shift => {
        Self::is_key_down_raw(VK_LSHIFT.0)
          || Self::is_key_down_raw(VK_RSHIFT.0)
      }
      _ => {
        if let Ok(key_code) = KeyCode::try_from(key) {
          Self::is_key_down_raw(key_code.0)
        } else {
          false
        }
      }
    }
  }

  /// Gets whether the specified key is currently down using the raw key
  /// code.
  fn is_key_down_raw(key: u16) -> bool {
    unsafe { (GetKeyState(key.into()) & 0x80) == 0x80 }
  }
}

/// Wrapper for the low-level keyboard hook API.
#[derive(Debug)]
pub struct KeyboardHook {
  handle: HHOOK,
  dispatcher: Dispatcher,
}

impl KeyboardHook {
  /// Creates a new low-level keyboard hook for the dispatcher's thread.
  ///
  /// The callback is called for every keyboard event and returns `true` if
  /// the event should be intercepted.
  ///
  /// # Panics
  ///
  /// Panics when attempting to register multiple hooks on the dispatcher's
  /// thread.
  pub fn new<F>(
    callback: F,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self>
  where
    F: Fn(KeyEvent) -> bool + Send + Sync + 'static,
  {
    let handle = dispatcher.dispatch_sync(move || {
      HOOK.with(|state| {
        assert!(
          state.take().is_none(),
          "Only one keyboard hook can be registered on the dispatcher's thread."
        );

        state.set(Some(Box::new(callback)));
      });

      unsafe {
        SetWindowsHookExW(
          WH_KEYBOARD_LL,
          Some(Self::hook_proc),
          HINSTANCE::default(),
          0,
        )
      }
    })??;

    Ok(Self {
      handle,
      dispatcher: dispatcher.clone(),
    })
  }

  /// Hook procedure for keyboard events.
  ///
  /// For use with `SetWindowsHookExW`.
  extern "system" fn hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {
    // If the code is less than zero, the hook procedure must pass the hook
    // notification directly to other applications.
    if code != 0 {
      return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    // Get struct with the keyboard input event.
    let input = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };

    let key_code = KeyCode(input.vkCode as u16);
    let is_keydown =
      wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN;

    let Ok(key) = Key::try_from(key_code) else {
      return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    };

    let key_event = KeyEvent::new(key, key_code, is_keydown);

    let should_intercept = HOOK.with(|state| {
      if let Some(callback) = state.take() {
        let result = callback(key_event);
        state.set(Some(callback));
        result
      } else {
        false
      }
    });

    if should_intercept {
      return LRESULT(1);
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
  }

  /// Terminates the keyboard hook by unregistering it.
  pub fn terminate(&mut self) -> crate::Result<()> {
    unsafe { UnhookWindowsHookEx(self.handle) }?;

    // Dispatch cleanup to the event loop thread since the callback
    // is stored in a thread-local on that thread.
    let _ = self.dispatcher.dispatch_async(|| {
      HOOK.with(|state| {
        state.take();
      });
    });

    Ok(())
  }
}

impl Drop for KeyboardHook {
  fn drop(&mut self) {
    let _ = self.terminate();
  }
}
