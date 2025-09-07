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

use crate::{Dispatcher, Key};

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
  /// Whether the event is for a key press or release.
  pub is_keypress: bool,

  /// The key that was pressed or released.
  pub key: Key,

  /// Virtual key code of the pressed key.
  vk_code: u16,
}

impl KeyEvent {
  /// Creates an instance of `KeyEvent`.
  pub(crate) fn new(key: Key, is_keypress: bool, vk_code: u16) -> Self {
    Self {
      is_keypress,
      key,
      vk_code,
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
        if let Some(vk_code) = key_to_vk_code(key) {
          Self::is_key_down_raw(vk_code)
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

/// Convert Windows virtual key code to cross-platform Key enum.
///
/// Returns `None` for unsupported key codes.
fn vk_code_to_key(vk_code: u16) -> Option<Key> {
  match VIRTUAL_KEY(vk_code) {
    VK_A => Some(Key::A),
    VK_B => Some(Key::B),
    VK_C => Some(Key::C),
    VK_D => Some(Key::D),
    VK_E => Some(Key::E),
    VK_F => Some(Key::F),
    VK_G => Some(Key::G),
    VK_H => Some(Key::H),
    VK_I => Some(Key::I),
    VK_J => Some(Key::J),
    VK_K => Some(Key::K),
    VK_L => Some(Key::L),
    VK_M => Some(Key::M),
    VK_N => Some(Key::N),
    VK_O => Some(Key::O),
    VK_P => Some(Key::P),
    VK_Q => Some(Key::Q),
    VK_R => Some(Key::R),
    VK_S => Some(Key::S),
    VK_T => Some(Key::T),
    VK_U => Some(Key::U),
    VK_V => Some(Key::V),
    VK_W => Some(Key::W),
    VK_X => Some(Key::X),
    VK_Y => Some(Key::Y),
    VK_Z => Some(Key::Z),
    VK_0 => Some(Key::D0),
    VK_1 => Some(Key::D1),
    VK_2 => Some(Key::D2),
    VK_3 => Some(Key::D3),
    VK_4 => Some(Key::D4),
    VK_5 => Some(Key::D5),
    VK_6 => Some(Key::D6),
    VK_7 => Some(Key::D7),
    VK_8 => Some(Key::D8),
    VK_9 => Some(Key::D9),
    VK_F1 => Some(Key::F1),
    VK_F2 => Some(Key::F2),
    VK_F3 => Some(Key::F3),
    VK_F4 => Some(Key::F4),
    VK_F5 => Some(Key::F5),
    VK_F6 => Some(Key::F6),
    VK_F7 => Some(Key::F7),
    VK_F8 => Some(Key::F8),
    VK_F9 => Some(Key::F9),
    VK_F10 => Some(Key::F10),
    VK_F11 => Some(Key::F11),
    VK_F12 => Some(Key::F12),
    VK_F13 => Some(Key::F13),
    VK_F14 => Some(Key::F14),
    VK_F15 => Some(Key::F15),
    VK_F16 => Some(Key::F16),
    VK_F17 => Some(Key::F17),
    VK_F18 => Some(Key::F18),
    VK_F19 => Some(Key::F19),
    VK_F20 => Some(Key::F20),
    VK_F21 => Some(Key::F21),
    VK_F22 => Some(Key::F22),
    VK_F23 => Some(Key::F23),
    VK_F24 => Some(Key::F24),
    VK_LWIN => Some(Key::LCmd),
    VK_RWIN => Some(Key::RCmd),
    VK_LCONTROL => Some(Key::LCtrl),
    VK_RCONTROL => Some(Key::RCtrl),
    VK_LMENU => Some(Key::LAlt),
    VK_RMENU => Some(Key::RAlt),
    VK_LSHIFT => Some(Key::LShift),
    VK_RSHIFT => Some(Key::RShift),
    VK_SPACE => Some(Key::Space),
    VK_TAB => Some(Key::Tab),
    VK_RETURN => Some(Key::Enter),
    VK_DELETE => Some(Key::Delete),
    VK_ESCAPE => Some(Key::Escape),
    VK_BACK => Some(Key::Backspace),
    VK_LEFT => Some(Key::Left),
    VK_RIGHT => Some(Key::Right),
    VK_UP => Some(Key::Up),
    VK_DOWN => Some(Key::Down),
    VK_HOME => Some(Key::Home),
    VK_END => Some(Key::End),
    VK_PRIOR => Some(Key::PageUp),
    VK_NEXT => Some(Key::PageDown),
    VK_INSERT => Some(Key::Insert),
    VK_OEM_1 => Some(Key::Semicolon),
    VK_OEM_7 => Some(Key::Quote),
    VK_OEM_COMMA => Some(Key::Comma),
    VK_OEM_PERIOD => Some(Key::Period),
    VK_OEM_2 => Some(Key::Slash),
    VK_OEM_5 => Some(Key::Backslash),
    VK_OEM_4 => Some(Key::LeftBracket),
    VK_OEM_6 => Some(Key::RightBracket),
    VK_OEM_MINUS => Some(Key::Minus),
    VK_OEM_PLUS => Some(Key::Equal),
    VK_OEM_3 => Some(Key::Grave),
    VK_NUMPAD0 => Some(Key::Numpad0),
    VK_NUMPAD1 => Some(Key::Numpad1),
    VK_NUMPAD2 => Some(Key::Numpad2),
    VK_NUMPAD3 => Some(Key::Numpad3),
    VK_NUMPAD4 => Some(Key::Numpad4),
    VK_NUMPAD5 => Some(Key::Numpad5),
    VK_NUMPAD6 => Some(Key::Numpad6),
    VK_NUMPAD7 => Some(Key::Numpad7),
    VK_NUMPAD8 => Some(Key::Numpad8),
    VK_NUMPAD9 => Some(Key::Numpad9),
    VK_ADD => Some(Key::NumpadAdd),
    VK_SUBTRACT => Some(Key::NumpadSubtract),
    VK_MULTIPLY => Some(Key::NumpadMultiply),
    VK_DIVIDE => Some(Key::NumpadDivide),
    VK_DECIMAL => Some(Key::NumpadDecimal),
    _ => None,
  }
}

/// Convert cross-platform Key enum to Windows virtual key code.
///
/// Returns `None` for unsupported keys. Generic modifier keys are mapped
/// to their left variants (e.g., `Key::Cmd` maps to `VK_LWIN`).
fn key_to_vk_code(key: Key) -> Option<u16> {
  match key {
    Key::A => Some(VK_A.0),
    Key::B => Some(VK_B.0),
    Key::C => Some(VK_C.0),
    Key::D => Some(VK_D.0),
    Key::E => Some(VK_E.0),
    Key::F => Some(VK_F.0),
    Key::G => Some(VK_G.0),
    Key::H => Some(VK_H.0),
    Key::I => Some(VK_I.0),
    Key::J => Some(VK_J.0),
    Key::K => Some(VK_K.0),
    Key::L => Some(VK_L.0),
    Key::M => Some(VK_M.0),
    Key::N => Some(VK_N.0),
    Key::O => Some(VK_O.0),
    Key::P => Some(VK_P.0),
    Key::Q => Some(VK_Q.0),
    Key::R => Some(VK_R.0),
    Key::S => Some(VK_S.0),
    Key::T => Some(VK_T.0),
    Key::U => Some(VK_U.0),
    Key::V => Some(VK_V.0),
    Key::W => Some(VK_W.0),
    Key::X => Some(VK_X.0),
    Key::Y => Some(VK_Y.0),
    Key::Z => Some(VK_Z.0),
    Key::D0 => Some(VK_0.0),
    Key::D1 => Some(VK_1.0),
    Key::D2 => Some(VK_2.0),
    Key::D3 => Some(VK_3.0),
    Key::D4 => Some(VK_4.0),
    Key::D5 => Some(VK_5.0),
    Key::D6 => Some(VK_6.0),
    Key::D7 => Some(VK_7.0),
    Key::D8 => Some(VK_8.0),
    Key::D9 => Some(VK_9.0),
    Key::F1 => Some(VK_F1.0),
    Key::F2 => Some(VK_F2.0),
    Key::F3 => Some(VK_F3.0),
    Key::F4 => Some(VK_F4.0),
    Key::F5 => Some(VK_F5.0),
    Key::F6 => Some(VK_F6.0),
    Key::F7 => Some(VK_F7.0),
    Key::F8 => Some(VK_F8.0),
    Key::F9 => Some(VK_F9.0),
    Key::F10 => Some(VK_F10.0),
    Key::F11 => Some(VK_F11.0),
    Key::F12 => Some(VK_F12.0),
    Key::F13 => Some(VK_F13.0),
    Key::F14 => Some(VK_F14.0),
    Key::F15 => Some(VK_F15.0),
    Key::F16 => Some(VK_F16.0),
    Key::F17 => Some(VK_F17.0),
    Key::F18 => Some(VK_F18.0),
    Key::F19 => Some(VK_F19.0),
    Key::F20 => Some(VK_F20.0),
    Key::F21 => Some(VK_F21.0),
    Key::F22 => Some(VK_F22.0),
    Key::F23 => Some(VK_F23.0),
    Key::F24 => Some(VK_F24.0),
    Key::LCmd => Some(VK_LWIN.0),
    Key::RCmd => Some(VK_RWIN.0),
    Key::LCtrl => Some(VK_LCONTROL.0),
    Key::RCtrl => Some(VK_RCONTROL.0),
    Key::LAlt => Some(VK_LMENU.0),
    Key::RAlt => Some(VK_RMENU.0),
    Key::LShift => Some(VK_LSHIFT.0),
    Key::RShift => Some(VK_RSHIFT.0),
    Key::Space => Some(VK_SPACE.0),
    Key::Tab => Some(VK_TAB.0),
    Key::Enter | Key::Return => Some(VK_RETURN.0),
    Key::Delete => Some(VK_DELETE.0),
    Key::Escape => Some(VK_ESCAPE.0),
    Key::Backspace => Some(VK_BACK.0),
    Key::Left => Some(VK_LEFT.0),
    Key::Right => Some(VK_RIGHT.0),
    Key::Up => Some(VK_UP.0),
    Key::Down => Some(VK_DOWN.0),
    Key::Home => Some(VK_HOME.0),
    Key::End => Some(VK_END.0),
    Key::PageUp => Some(VK_PRIOR.0),
    Key::PageDown => Some(VK_NEXT.0),
    Key::Insert => Some(VK_INSERT.0),
    Key::Semicolon => Some(VK_OEM_1.0),
    Key::Quote => Some(VK_OEM_7.0),
    Key::Comma => Some(VK_OEM_COMMA.0),
    Key::Period => Some(VK_OEM_PERIOD.0),
    Key::Slash => Some(VK_OEM_2.0),
    Key::Backslash => Some(VK_OEM_5.0),
    Key::LeftBracket => Some(VK_OEM_4.0),
    Key::RightBracket => Some(VK_OEM_6.0),
    Key::Minus => Some(VK_OEM_MINUS.0),
    Key::Equal => Some(VK_OEM_PLUS.0),
    Key::Grave => Some(VK_OEM_3.0),
    Key::Numpad0 => Some(VK_NUMPAD0.0),
    Key::Numpad1 => Some(VK_NUMPAD1.0),
    Key::Numpad2 => Some(VK_NUMPAD2.0),
    Key::Numpad3 => Some(VK_NUMPAD3.0),
    Key::Numpad4 => Some(VK_NUMPAD4.0),
    Key::Numpad5 => Some(VK_NUMPAD5.0),
    Key::Numpad6 => Some(VK_NUMPAD6.0),
    Key::Numpad7 => Some(VK_NUMPAD7.0),
    Key::Numpad8 => Some(VK_NUMPAD8.0),
    Key::Numpad9 => Some(VK_NUMPAD9.0),
    Key::NumpadAdd => Some(VK_ADD.0),
    Key::NumpadSubtract => Some(VK_SUBTRACT.0),
    Key::NumpadMultiply => Some(VK_MULTIPLY.0),
    Key::NumpadDivide => Some(VK_DIVIDE.0),
    Key::NumpadDecimal => Some(VK_DECIMAL.0),
    // Generic modifier keys map to their left variants.
    Key::Cmd | Key::Win => Some(VK_LWIN.0),
    Key::Ctrl => Some(VK_LCONTROL.0),
    Key::Alt => Some(VK_LMENU.0),
    Key::Shift => Some(VK_LSHIFT.0),
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

    let vk_code = input.vkCode as u16;
    let is_keydown =
      wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN;

    if let Some(key) = vk_code_to_key(vk_code) {
      let key_event = KeyEvent::new(key, is_keydown, vk_code);

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
