use std::{cell::Cell, ptr};

use windows::Win32::{
  Foundation::{LPARAM, LRESULT, WPARAM},
  UI::{
    Input::KeyboardAndMouse::{
      GetKeyState, VIRTUAL_KEY, VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6,
      VK_7, VK_8, VK_9, VK_A, VK_ADD, VK_B, VK_BACK, VK_C, VK_D,
      VK_DECIMAL, VK_DELETE, VK_DIVIDE, VK_DOWN, VK_E, VK_END, VK_ESCAPE,
      VK_F, VK_F1, VK_F10, VK_F11, VK_F12, VK_F13, VK_F14, VK_F15, VK_F16,
      VK_F17, VK_F18, VK_F19, VK_F2, VK_F20, VK_F21, VK_F22, VK_F23,
      VK_F24, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_G, VK_H,
      VK_HOME, VK_I, VK_INSERT, VK_J, VK_K, VK_L, VK_LCONTROL, VK_LEFT,
      VK_LMENU, VK_LSHIFT, VK_LWIN, VK_M, VK_MULTIPLY, VK_N, VK_NEXT,
      VK_NUMPAD0, VK_NUMPAD1, VK_NUMPAD2, VK_NUMPAD3, VK_NUMPAD4,
      VK_NUMPAD5, VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8, VK_NUMPAD9, VK_O,
      VK_OEM_1, VK_OEM_2, VK_OEM_3, VK_OEM_4, VK_OEM_5, VK_OEM_6,
      VK_OEM_7, VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS,
      VK_P, VK_PRIOR, VK_Q, VK_R, VK_RCONTROL, VK_RETURN, VK_RIGHT,
      VK_RMENU, VK_RSHIFT, VK_RWIN, VK_S, VK_SPACE, VK_SUBTRACT, VK_T,
      VK_TAB, VK_U, VK_UP, VK_V, VK_W, VK_X, VK_Y, VK_Z,
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
  static HOOK: Cell<Option<Box<dyn FnMut(KeyEvent) -> bool>>> = Cell::default();
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
        let key_code = KeyCode::from(key);
        Self::is_key_down_raw(key_code.0)
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
}

impl KeyboardHook {
  /// Creates a new low-level keyboard hook for the main thread.
  ///
  /// # Panics
  ///
  /// Panics when attempting to register multiple hooks.
  #[must_use]
  pub fn new<F>(dispatcher: Dispatcher, callback: F) -> crate::Result<Self>
  where
    F: FnMut(KeyEvent) -> bool + 'static,
  {
    let handle = dispatcher.dispatch_sync(move || {
      HOOK.with(|state| {
        assert!(
          state.take().is_none(),
          "Only one keyboard hook can be registered on the main thread."
        );

        state.set(Some(Box::new(callback)));

        unsafe {
          SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(Self::hook_proc),
            ptr::null_mut(),
            0,
          )
        }
      })??;

      Ok(KeyboardHook { handle })
    });
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

    let key_event =
      KeyEvent::new(Key::from(key_code), key_code, is_keydown);

    let should_intercept = HOOK.with(|state| {
      if let Some(mut callback) = state.take() {
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

  /// Stops the keyboard hook by unregistering it.
  pub fn stop(&mut self) -> crate::Result<()> {
    unsafe { UnhookWindowsHookEx(self.handle) }?;
    HOOK.with(|state| state.take());
    Ok(())
  }
}

impl Drop for KeyboardHook {
  fn drop(&mut self) {
    let _ = self.stop();
  }
}
