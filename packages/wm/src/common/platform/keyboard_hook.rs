use std::{cell::OnceCell, collections::HashMap};

use tokio::sync::mpsc;
use tracing::warn;
use windows::Win32::{
  Foundation::{LPARAM, LRESULT, WPARAM},
  UI::{
    Input::KeyboardAndMouse::*,
    WindowsAndMessaging::{
      CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
      KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
    },
  },
};

use crate::user_config::KeybindingConfig;

use super::PlatformEvent;

thread_local! {
  /// Thread-local for instance of `KeyboardHook`.
  ///
  /// For use with hook procedure.
  static KEYBOARD_HOOK: OnceCell<KeyboardHook> = OnceCell::new();
}

/// Available modifier keys.
const MODIFIER_KEYS: [u16; 6] = [
  VK_LSHIFT.0,
  VK_RSHIFT.0,
  VK_LCONTROL.0,
  VK_RCONTROL.0,
  VK_LMENU.0,
  VK_RMENU.0,
];

pub fn set_local_keyboard_hook(hook: KeyboardHook) {
  let _ = KEYBOARD_HOOK.with(|cell| cell.set(hook));
}

pub struct ActiveKeybinding {
  pub vk_codes: Vec<u16>,
  pub config: KeybindingConfig,
}

pub struct KeyboardHook {
  /// Sender to emit platform events.
  event_tx: mpsc::UnboundedSender<PlatformEvent>,

  /// Handle to the keyboard hook.
  hook: HHOOK,

  /// Active keybindings grouped by trigger key. The trigger key is the
  /// final key in a key combination.
  keybindings_by_trigger_key: HashMap<u16, Vec<ActiveKeybinding>>,
}

impl KeyboardHook {
  pub fn new() -> Self {
    todo!()
  }

  /// Starts a keyboard hook on the current thread.
  ///
  /// Assumes that a message loop is currently running.
  pub fn start(
    keybindings: Vec<KeybindingConfig>,
    event_tx: mpsc::UnboundedSender<PlatformEvent>,
  ) -> anyhow::Result<Self> {
    // Register low-level keyboard hook.
    let hook = unsafe {
      SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0)?
    };

    Ok(Self {
      event_tx,
      hook,
      keybindings_by_trigger_key: Self::keybindings_by_trigger_key(
        keybindings,
      ),
    })
  }

  pub fn update(&mut self, keybindings: Vec<KeybindingConfig>) {
    self.keybindings_by_trigger_key =
      Self::keybindings_by_trigger_key(keybindings);
  }

  pub fn stop(&mut self) {
    self.keybindings_by_trigger_key.clear();
    let _ = unsafe { UnhookWindowsHookEx(self.hook) };
  }

  fn keybindings_by_trigger_key(
    keybindings: Vec<KeybindingConfig>,
  ) -> HashMap<u16, Vec<ActiveKeybinding>> {
    let mut keybinding_map = HashMap::new();

    for keybinding in &keybindings {
      for binding in &keybinding.bindings {
        let vk_codes = binding
          .split("+")
          .filter_map(|key| {
            let vk_code = Self::key_to_vk_code(key);

            if vk_code.is_none() {
              warn!("Unrecognized key on current keyboard '{}'.", key);
            }

            vk_code
          })
          .collect::<Vec<_>>();

        // Safety: A split string always has at least one element.
        let trigger_key = vk_codes.last().unwrap().clone();

        keybinding_map
          .entry(trigger_key)
          .or_insert_with(|| Vec::new()) // vec only created if needed.
          .push(ActiveKeybinding {
            vk_codes,
            config: keybinding.clone(),
          });
      }
    }

    keybinding_map
  }

  fn key_to_vk_code(key: &str) -> Option<u16> {
    match key.to_lowercase().as_str() {
      "a" => Some(VK_A.0),
      "b" => Some(VK_B.0),
      "c" => Some(VK_C.0),
      "d" => Some(VK_D.0),
      "e" => Some(VK_E.0),
      "f" => Some(VK_F.0),
      "g" => Some(VK_G.0),
      "h" => Some(VK_H.0),
      "i" => Some(VK_I.0),
      "j" => Some(VK_J.0),
      "k" => Some(VK_K.0),
      "l" => Some(VK_L.0),
      "m" => Some(VK_M.0),
      "n" => Some(VK_N.0),
      "o" => Some(VK_O.0),
      "p" => Some(VK_P.0),
      "q" => Some(VK_Q.0),
      "r" => Some(VK_R.0),
      "s" => Some(VK_S.0),
      "t" => Some(VK_T.0),
      "u" => Some(VK_U.0),
      "v" => Some(VK_V.0),
      "w" => Some(VK_W.0),
      "x" => Some(VK_X.0),
      "y" => Some(VK_Y.0),
      "z" => Some(VK_Z.0),
      "d0" | "0" => Some(VK_0.0),
      "d1" | "1" => Some(VK_1.0),
      "d2" | "2" => Some(VK_2.0),
      "d3" | "3" => Some(VK_3.0),
      "d4" | "4" => Some(VK_4.0),
      "d5" | "5" => Some(VK_5.0),
      "d6" | "6" => Some(VK_6.0),
      "d7" | "7" => Some(VK_7.0),
      "d8" | "8" => Some(VK_8.0),
      "d9" | "9" => Some(VK_9.0),
      "shiftkey" | "shift" => Some(VK_SHIFT.0),
      "controlkey" | "control" => Some(VK_CONTROL.0),
      "lshiftkey" | "lshift" => Some(VK_LSHIFT.0),
      "rshiftkey" | "rshift" => Some(VK_RSHIFT.0),
      "menu" | "alt" => Some(VK_MENU.0),
      "win" | "lwin" => Some(VK_LWIN.0),
      "rwin" => Some(VK_RWIN.0),
      "lcontrolkey" | "lcontrol" | "lctrl" => Some(VK_LCONTROL.0),
      "rcontrolkey" | "rcontrol" | "rctrl" => Some(VK_RCONTROL.0),
      "lmenu" | "lalt" => Some(VK_LMENU.0),
      "rmenu" | "ralt" => Some(VK_RMENU.0),
      "back" => Some(VK_BACK.0),
      "tab" => Some(VK_TAB.0),
      "clear" => Some(VK_CLEAR.0),
      "enter" | "return" => Some(VK_RETURN.0),
      "pause" => Some(VK_PAUSE.0),
      "capital" | "capslock" => Some(VK_CAPITAL.0),
      "kanamode" | "hangulmode" => Some(VK_KANA.0),
      "junjamode" => Some(VK_JUNJA.0),
      "finalmode" => Some(VK_FINAL.0),
      "hanjamode" | "kanjimode" => Some(VK_HANJA.0),
      "escape" => Some(VK_ESCAPE.0),
      "imeconvert" => Some(VK_CONVERT.0),
      "imenonconvert" => Some(VK_NONCONVERT.0),
      "imeaccept" => Some(VK_ACCEPT.0),
      "imemodechange" => Some(VK_MODECHANGE.0),
      "space" => Some(VK_SPACE.0),
      "prior" | "pageup" => Some(VK_PRIOR.0),
      "next" | "pagedown" => Some(VK_NEXT.0),
      "end" => Some(VK_END.0),
      "home" => Some(VK_HOME.0),
      "left" => Some(VK_LEFT.0),
      "up" => Some(VK_UP.0),
      "right" => Some(VK_RIGHT.0),
      "down" => Some(VK_DOWN.0),
      "select" => Some(VK_SELECT.0),
      "print" => Some(VK_PRINT.0),
      "execute" => Some(VK_EXECUTE.0),
      "snapshot" | "printscreen" => Some(VK_SNAPSHOT.0),
      "insert" => Some(VK_INSERT.0),
      "delete" => Some(VK_DELETE.0),
      "help" => Some(VK_HELP.0),
      "apps" => Some(VK_APPS.0),
      "sleep" => Some(VK_SLEEP.0),
      "numpad0" => Some(VK_NUMPAD0.0),
      "numpad1" => Some(VK_NUMPAD1.0),
      "numpad2" => Some(VK_NUMPAD2.0),
      "numpad3" => Some(VK_NUMPAD3.0),
      "numpad4" => Some(VK_NUMPAD4.0),
      "numpad5" => Some(VK_NUMPAD5.0),
      "numpad6" => Some(VK_NUMPAD6.0),
      "numpad7" => Some(VK_NUMPAD7.0),
      "numpad8" => Some(VK_NUMPAD8.0),
      "numpad9" => Some(VK_NUMPAD9.0),
      "multiply" => Some(VK_MULTIPLY.0),
      "add" => Some(VK_ADD.0),
      "separator" => Some(VK_SEPARATOR.0),
      "subtract" => Some(VK_SUBTRACT.0),
      "decimal" => Some(VK_DECIMAL.0),
      "divide" => Some(VK_DIVIDE.0),
      "f1" => Some(VK_F1.0),
      "f2" => Some(VK_F2.0),
      "f3" => Some(VK_F3.0),
      "f4" => Some(VK_F4.0),
      "f5" => Some(VK_F5.0),
      "f6" => Some(VK_F6.0),
      "f7" => Some(VK_F7.0),
      "f8" => Some(VK_F8.0),
      "f9" => Some(VK_F9.0),
      "f10" => Some(VK_F10.0),
      "f11" => Some(VK_F11.0),
      "f12" => Some(VK_F12.0),
      "f13" => Some(VK_F13.0),
      "f14" => Some(VK_F14.0),
      "f15" => Some(VK_F15.0),
      "f16" => Some(VK_F16.0),
      "f17" => Some(VK_F17.0),
      "f18" => Some(VK_F18.0),
      "f19" => Some(VK_F19.0),
      "f20" => Some(VK_F20.0),
      "f21" => Some(VK_F21.0),
      "f22" => Some(VK_F22.0),
      "f23" => Some(VK_F23.0),
      "f24" => Some(VK_F24.0),
      "numlock" => Some(VK_NUMLOCK.0),
      "scroll" => Some(VK_SCROLL.0),
      "browserback" => Some(VK_BROWSER_BACK.0),
      "browserforward" => Some(VK_BROWSER_FORWARD.0),
      "browserrefresh" => Some(VK_BROWSER_REFRESH.0),
      "browserstop" => Some(VK_BROWSER_STOP.0),
      "browsersearch" => Some(VK_BROWSER_SEARCH.0),
      "browserfavorites" => Some(VK_BROWSER_FAVORITES.0),
      "browserhome" => Some(VK_BROWSER_HOME.0),
      "volumemute" => Some(VK_VOLUME_MUTE.0),
      "volumedown" => Some(VK_VOLUME_DOWN.0),
      "volumeup" => Some(VK_VOLUME_UP.0),
      "medianexttrack" => Some(VK_MEDIA_NEXT_TRACK.0),
      "mediaprevioustrack" => Some(VK_MEDIA_PREV_TRACK.0),
      "mediastop" => Some(VK_MEDIA_STOP.0),
      "mediaplaypause" => Some(VK_MEDIA_PLAY_PAUSE.0),
      "launchmail" => Some(VK_LAUNCH_MAIL.0),
      "selectmedia" => Some(VK_LAUNCH_MEDIA_SELECT.0),
      "launchapplication1" | "launchapp1" => Some(VK_LAUNCH_APP1.0),
      "launchapplication2" | "launchapp2" => Some(VK_LAUNCH_APP2.0),
      "oem1" | "oemsemicolon" => Some(VK_OEM_1.0),
      "oemplus" => Some(VK_OEM_PLUS.0),
      "oemcomma" => Some(VK_OEM_COMMA.0),
      "oemminus" => Some(VK_OEM_MINUS.0),
      "oemperiod" => Some(VK_OEM_PERIOD.0),
      "oem2" | "oemquestion" => Some(VK_OEM_2.0),
      "oem3" | "oemtilde" => Some(VK_OEM_3.0),
      "oem4" | "oemopenbrackets" => Some(VK_OEM_4.0),
      "oem5" | "oempipe" => Some(VK_OEM_5.0),
      "oem6" | "oemclosebrackets" => Some(VK_OEM_6.0),
      "oem7" | "oemquotes" => Some(VK_OEM_7.0),
      "oem8" => Some(VK_OEM_8.0),
      "oembackslash" | "oembacktab" => Some(VK_OEM_BACKTAB.0),
      "oem102" => Some(VK_OEM_102.0),
      "processkey" => Some(VK_PROCESSKEY.0),
      "packet" => Some(VK_PACKET.0),
      "attn" => Some(VK_ATTN.0),
      "crsel" => Some(VK_CRSEL.0),
      "exsel" => Some(VK_EXSEL.0),
      "eraseeof" => Some(VK_EREOF.0),
      "play" => Some(VK_PLAY.0),
      "zoom" => Some(VK_ZOOM.0),
      "pa1" => Some(VK_PA1.0),
      "oemclear" => Some(VK_OEM_CLEAR.0),
      _ => {
        // TODO: Check if the key exists on the current keyboard layout.
        // let xx = unsafe { GetKeyboardLayout(0) };
        // unsafe { VkKeyScanExW(key, xx) as _ }
        None
      }
    }
  }

  /// Emits a platform event if a keybinding should be triggered.
  ///
  /// Returns `true` if the event should be blocked and not sent to other
  /// applications.
  fn handle_key_event(&self, vk_code: u16) -> bool {
    println!("Key event: {}", vk_code);

    match self.keybindings_by_trigger_key.get(&vk_code) {
      // Forward the event if no keybindings exist for the trigger key.
      None => false,
      // Otherwise, check if there is a matching keybinding.
      Some(keybindings) => {
        // let mut cached_key_states = HashMap::new();

        // // Find the matching keybindings based on the pressed keys.
        // let matched_keybindings =
        //   keybindings.iter().filter(|keybinding| {
        //     keybinding.vk_codes.iter().all(|&key| {
        //       if key == vk_code {
        //         return true;
        //       }

        //       if let Some(&is_key_down) = cached_key_states.get(&key) {
        //         return is_key_down;
        //       }

        //       let is_key_down = Self::is_key_down(key);
        //       cached_key_states.insert(key, is_key_down);
        //       is_key_down
        //     })
        //   });

        // // Find the longest matching keybinding.
        // let longest_keybinding = matched_keybindings
        //   .max_by_key(|keybinding| keybinding.vk_codes.len());

        // if longest_keybinding.is_none() {
        //   return false;
        // }

        // let longest_keybinding = longest_keybinding.unwrap();

        // // Get the modifier keys to reject based on the longest matching keybinding.
        // let modifier_keys_to_reject =
        //   MODIFIER_KEYS.iter().filter(|&&modifier_key| {
        //     !longest_keybinding.vk_codes.contains(&modifier_key)
        //       && !longest_keybinding
        //         .vk_codes
        //         .contains(&Self::get_generic_key(modifier_key))
        //   });

        // // Check if any modifier keys to reject are currently down.
        // let has_modifier_keys_to_reject =
        //   modifier_keys_to_reject.any(|&modifier_key| {
        //     if let Some(&is_key_down) =
        //       cached_key_states.get(&modifier_key)
        //     {
        //       is_key_down
        //     } else {
        //       Self::is_key_down(modifier_key)
        //     }
        //   });

        // if has_modifier_keys_to_reject {
        //   return false;
        // }

        // // Invoke the callback function for the longest matching keybinding.
        // let _ = self.event_tx.send(PlatformEvent::KeybindingTriggered(
        //   longest_keybinding.clone(),
        // ));
        true
      }
    }
  }

  // /// Gets the generic key code for a given key code.
  // fn get_generic_key(key: u16) -> u16 {
  //   match key {
  //     VK_LMENU | VK_RMENU => VK_MENU,
  //     VK_LSHIFT | VK_RSHIFT => VK_SHIFT,
  //     VK_LCONTROL | VK_RCONTROL => VK_CONTROL,
  //     _ => key,
  //   }
  // }

  // /// Checks if the specified key is currently down.
  // fn is_key_down(key: u16) -> bool {
  //   match key {
  //     VK_MENU => {
  //       Self::is_key_down_raw(VK_LMENU) || Self::is_key_down_raw(VK_RMENU)
  //     }
  //     VK_SHIFT => {
  //       Self::is_key_down_raw(VK_LSHIFT)
  //         || Self::is_key_down_raw(VK_RSHIFT)
  //     }
  //     VK_CONTROL => {
  //       Self::is_key_down_raw(VK_LCONTROL)
  //         || Self::is_key_down_raw(VK_RCONTROL)
  //     }
  //     _ => Self::is_key_down_raw(key),
  //   }
  // }

  // /// Checks if the specified key is currently down using the raw key code.
  // fn is_key_down_raw(key: u16) -> bool {
  //   unsafe { (GetKeyState(key) & 0x8000) == 0x8000 }
  // }
}

extern "system" fn keyboard_hook_proc(
  code: i32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  let should_ignore = code != 0
    || !(wparam.0 as u32 == WM_KEYDOWN
      || wparam.0 as u32 == WM_SYSKEYDOWN);

  // If the code is less than zero, the hook procedure must pass the hook
  // notification directly to other applications. We also only care about
  // keydown events.
  if should_ignore {
    return unsafe { CallNextHookEx(None, code, wparam, lparam) };
  }

  // Get struct with keyboard input event.
  let input = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };

  KEYBOARD_HOOK.with(|hook| {
    if let Some(hook) = hook.get() {
      let should_block = hook.handle_key_event(input.vkCode as u16);

      if should_block {
        return LRESULT(1);
      }
    }

    return unsafe { CallNextHookEx(None, code, wparam, lparam) };
  })
}
