use std::{
  collections::HashMap,
  sync::{Arc, Mutex, OnceLock},
};

use tokio::sync::mpsc;
use tracing::warn;
use windows::Win32::{
  Foundation::{LPARAM, LRESULT, WPARAM},
  UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
    KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
  },
};
use wm_common::KeybindingConfig;

use super::PlatformEvent;
use crate::key::Key;

/// Global instance of `KeyboardHook`.
///
/// For use with hook procedure.
static KEYBOARD_HOOK: OnceLock<Arc<KeyboardHook>> = OnceLock::new();

/// Available modifier keys.
const MODIFIER_KEYS: [Key; 12] = [
  Key::LShift,
  Key::RShift,
  Key::Shift,
  Key::LControl,
  Key::RControl,
  Key::Control,
  Key::LAlt,
  Key::RAlt,
  Key::Alt,
  Key::LWin,
  Key::RWin,
  Key::Win,
];

#[derive(Debug)]
pub struct ActiveKeybinding {
  pub keys: Vec<Key>,
  pub config: KeybindingConfig,
}

#[derive(Debug)]
pub struct KeyboardHook {
  /// Sender to emit platform events.
  event_tx: mpsc::UnboundedSender<PlatformEvent>,

  /// Handle to the keyboard hook.
  hook: Arc<Mutex<HHOOK>>,

  /// Active keybindings grouped by trigger key. The trigger key is the
  /// final key in a key combination.
  keybindings_by_trigger_key:
    Arc<Mutex<HashMap<Key, Vec<ActiveKeybinding>>>>,
}

impl KeyboardHook {
  /// Creates an instance of `KeyboardHook`.
  pub fn new(
    keybindings: &Vec<KeybindingConfig>,
    event_tx: mpsc::UnboundedSender<PlatformEvent>,
  ) -> anyhow::Result<Arc<Self>> {
    let keyboard_hook = Arc::new(Self {
      event_tx,
      hook: Arc::new(Mutex::new(HHOOK::default())),
      keybindings_by_trigger_key: Arc::new(Mutex::new(
        Self::keybindings_by_trigger_key(keybindings),
      )),
    });

    KEYBOARD_HOOK
      .set(keyboard_hook.clone())
      .map_err(|_| anyhow::anyhow!("Keyboard hook already running."))?;

    Ok(keyboard_hook)
  }

  /// Starts a keyboard hook on the current thread.
  ///
  /// Assumes that a message loop is currently running.
  ///
  /// # Panics
  ///
  /// If the internal mutex is poisoned.
  pub fn start(&self) -> anyhow::Result<()> {
    *self.hook.lock().unwrap() = unsafe {
      SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_proc), None, 0)
    }?;

    Ok(())
  }

  /// Updates the keybindings for the keyboard hook.
  ///
  /// # Panics
  ///
  /// If the internal mutex is poisoned.
  pub fn update(&self, keybindings: &Vec<KeybindingConfig>) {
    *self.keybindings_by_trigger_key.lock().unwrap() =
      Self::keybindings_by_trigger_key(keybindings);
  }

  /// Stops the low-level keyboard hook.
  ///
  /// # Panics
  ///
  /// If the internal mutex is poisoned.
  pub fn stop(&self) -> anyhow::Result<()> {
    unsafe { UnhookWindowsHookEx(*self.hook.lock().unwrap()) }?;
    Ok(())
  }

  fn keybindings_by_trigger_key(
    keybindings: &Vec<KeybindingConfig>,
  ) -> HashMap<Key, Vec<ActiveKeybinding>> {
    let mut keybinding_map = HashMap::new();

    for keybinding in keybindings {
      for binding in &keybinding.bindings {
        let keys = binding
          .split('+')
          .filter_map(|key_str| {
            let key = Key::from_str(key_str);

            if key.is_none() {
              warn!(
                "Unrecognized key on current keyboard '{}'. Ensure that alt or shift isn't required for the key.",
                key_str
              );
            }

            key
          })
          .collect::<Vec<_>>();

        if let Some(trigger_key) = keys.last() {
          keybinding_map
            .entry(*trigger_key)
            .or_insert_with(Vec::new)
            .push(ActiveKeybinding {
              keys,
              config: keybinding.clone(),
            });
        }
      }
    }

    keybinding_map
  }

  /// Emits a platform event if a keybinding should be triggered.
  ///
  /// Returns `true` if the event should be blocked and not sent to other
  /// applications.
  fn handle_key_event(&self, vk_code: u16) -> bool {
    // TODO: Remove logging after more thourough testing.

    let pressed_key = Key::from_vk(vk_code);
    tracing::info!("Pressed key: {pressed_key:?}");
    match self
      .keybindings_by_trigger_key
      .lock()
      .unwrap()
      .get(&pressed_key)
    {
      // Forward the event if no keybindings exist for the trigger key.
      None => false,
      // Otherwise, check if there is a matching keybinding.
      Some(keybindings) => {
        let mut cached_key_states = HashMap::new();

        // Find the matching keybindings based on the pressed keys.
        let matched_keybindings =
          keybindings.iter().filter(|keybinding| {
            keybinding.keys.iter().all(|&key| {
              if key.is_analogous(pressed_key) {
                return true;
              }

              if let Some(&is_key_down) = cached_key_states.get(&key) {
                return is_key_down;
              }

              let is_key_down = key.is_down();
              cached_key_states.insert(key, is_key_down);
              is_key_down
            })
          });

        // Find the longest matching keybinding.
        let longest_keybinding = matched_keybindings
          .max_by_key(|keybinding| keybinding.keys.len());

        if longest_keybinding.is_none() {
          tracing::warn!("Didn't find keybind");
          return false;
        }

        let longest_keybinding = longest_keybinding.unwrap();

        tracing::info!(
          "Longest keybinding: {:?}",
          longest_keybinding.keys
        );

        // Get the modifier keys to reject based on the longest matching
        // keybinding.
        let modifier_keys_to_reject = MODIFIER_KEYS
          .iter()
          .filter(|&&modifier_key| {
            !longest_keybinding.keys.iter().any(|&key| {
              key.is_analogous(modifier_key)
                || modifier_key.is_analogous(key)
            })
          })
          .collect::<Vec<_>>();

        tracing::info!(
          "Modifier keys to reject: {:?}",
          modifier_keys_to_reject
        );

        // Check if any modifier keys to reject are currently down.
        let has_modifier_keys_to_reject =
          modifier_keys_to_reject.iter().any(|&modifier_key| {
            if let Some(&is_key_down) = cached_key_states.get(modifier_key)
            {
              if is_key_down {
                tracing::info!(
                  "Modifier key is down, rejecting: {modifier_key:?}"
                );
              }
              is_key_down
            } else {
              let is_key_down = modifier_key.is_down();
              if is_key_down {
                tracing::info!(
                  "Modifier key is down, rejecting: {modifier_key:?}"
                );
              }
              is_key_down
            }
          });

        if has_modifier_keys_to_reject {
          return false;
        }

        // Invoke the callback function for the longest matching
        // keybinding.
        let _ = self.event_tx.send(PlatformEvent::KeybindingTriggered(
          longest_keybinding.config.clone(),
        ));

        true
      }
    }
  }
}

extern "system" fn keyboard_hook_proc(
  code: i32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  #[allow(clippy::cast_possible_truncation)]
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

  if let Some(hook) = KEYBOARD_HOOK.get() {
    #[allow(clippy::cast_possible_truncation)]
    let should_block = hook.handle_key_event(input.vkCode as u16);

    if should_block {
      return LRESULT(1);
    }
  }

  unsafe { CallNextHookEx(None, code, wparam, lparam) }
}
