use std::{cell::OnceCell, collections::HashMap};

use tokio::sync::mpsc;
use tracing::warn;
use windows::Win32::{
  Foundation::{LPARAM, LRESULT, WPARAM},
  UI::{
    Input::KeyboardAndMouse::{GetKeyboardLayout, VkKeyScanExW, VK_4},
    WindowsAndMessaging::{
      CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
      KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
    },
  },
};

use crate::user_config::KeybindingConfig;

use super::PlatformEvent;

thread_local! {
  static KEYBOARD_HOOK: OnceCell<KeyboardHook> = OnceCell::new();
}

pub fn set_local_keyboard_hook(hook: KeyboardHook) {
  let _ = KEYBOARD_HOOK.with(|cell| cell.set(hook));
}

pub struct ActiveKeybinding {
  pub vk_codes: Vec<u32>,
  pub config: KeybindingConfig,
}

pub struct KeyboardHook {
  /// Sender to emit platform events.
  event_tx: mpsc::UnboundedSender<PlatformEvent>,

  /// Handle to the keyboard hook.
  hook: HHOOK,

  /// Active keybindings grouped by trigger key. The trigger key is the
  /// final key in a key combination.
  keybindings_by_trigger_key: HashMap<u32, Vec<ActiveKeybinding>>,
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
  ) -> HashMap<u32, Vec<ActiveKeybinding>> {
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

  fn key_to_vk_code(key: &str) -> Option<u32> {
    match key {
      "ctrl" => Some(0x11),
      "alt" => Some(0x12),
      "shift" => Some(0x10),
      "win" => Some(0x5B),
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
  /// Returns `true` if the event should be forwarded.
  fn handle_key_event(&self, vk_code: u32) -> bool {
    println!("Key event: {}", vk_code);

    match self.keybindings_by_trigger_key.get(&vk_code) {
      // Forward the event if no keybindings exist for the trigger key.
      None => true,
      Some(keybinding) => {
        // TODO: Emit platform event.
        // let _ = self.event_tx.send(keybinding.clone());
        false
      }
    }
  }
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
    let should_forward =
      hook.get().unwrap().handle_key_event(input.vkCode);

    if should_forward {
      unsafe { CallNextHookEx(None, code, wparam, lparam) }
    } else {
      LRESULT(1)
    }
  })
}
