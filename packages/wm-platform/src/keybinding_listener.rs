use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
};

use tokio::sync::mpsc;

use crate::{
  platform_event::KeybindingEvent, platform_impl, Dispatcher, Key,
};

/// Modifier key groups, where each entry maps a generic key (e.g.
/// `Key::Shift`) to all its variants (e.g. `Key::LShift`, `Key::RShift`).
///
/// `Cmd` and `Win` are treated as aliases within the same group.
const MODIFIER_GROUPS: &[(Key, &[Key])] = &[
  (Key::Shift, &[Key::Shift, Key::LShift, Key::RShift]),
  (Key::Ctrl, &[Key::Ctrl, Key::LCtrl, Key::RCtrl]),
  (Key::Alt, &[Key::Alt, Key::LAlt, Key::RAlt]),
  (
    Key::Win,
    &[
      Key::Win,
      Key::LWin,
      Key::RWin,
      Key::Cmd,
      Key::LCmd,
      Key::RCmd,
    ],
  ),
];

/// Maps a key to its canonical form.
///
/// Side-specific and aliased modifier keys (e.g. `Key::LWin`, `Key::Cmd`)
/// are collapsed to the "generic" key of their modifier group (e.g.
/// `Key::Win`). Non-modifier keys are returned unchanged.
///
/// This is needed because the OS reports modifier presses using a specific
/// virtual key (e.g. `VK_LWIN`), which our decoding resolves to a single
/// `Key` variant, whereas user configs may specify a different variant of
/// the same modifier (e.g. `lwin` vs `win`). Canonicalizing both sides
/// ensures they match.
#[must_use]
fn canonical_key(key: Key) -> Key {
  for (generic_key, group_keys) in MODIFIER_GROUPS {
    if group_keys.contains(&key) {
      return *generic_key;
    }
  }

  key
}

/// Returns whether the given key is a modifier key.
#[must_use]
fn is_modifier(key: Key) -> bool {
  MODIFIER_GROUPS
    .iter()
    .any(|(_, group_keys)| group_keys.contains(&key))
}

/// The set of keybinding lookup maps used by the keyboard hook.
#[derive(Debug, Default)]
struct KeybindingMaps {
  /// Regular keybindings, keyed by the canonical form of their trigger
  /// key. These fire on key press.
  regular: HashMap<Key, Vec<Keybinding>>,

  /// "Tap" keybindings - single-key bindings consisting only of a
  /// modifier key (e.g. `lwin`). Keyed by the canonical form of that key.
  ///
  /// These fire on key *release*, but only if the modifier was tapped on
  /// its own (i.e. no other key was pressed while it was held). This
  /// allows binding a lone modifier without swallowing its key press,
  /// which would otherwise break key combinations (e.g. `lwin+a`) and
  /// native shortcuts.
  taps: HashMap<Key, Keybinding>,
}

/// Tracks the in-progress state of a potential modifier "tap".
#[derive(Debug, Default)]
struct TapState {
  /// The canonical modifier key currently held as a tap candidate, if any.
  pending: Option<Key>,

  /// Whether another key was pressed while the candidate was held, which
  /// disqualifies it from firing as a tap.
  dirty: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Keybinding(Vec<Key>);

impl Keybinding {
  /// Creates a new keybinding from a vector of keys.
  ///
  /// # Errors
  ///
  /// Returns [`Error::InvalidKeybinding`] if the keybinding is empty.
  pub fn new(keys: Vec<Key>) -> crate::Result<Self> {
    if keys.is_empty() {
      return Err(crate::Error::InvalidKeybinding);
    }

    Ok(Self(keys))
  }

  /// Returns the keys in the keybinding.
  #[must_use]
  pub fn keys(&self) -> &[Key] {
    &self.0
  }

  /// Returns the trigger key in the keybinding.
  #[must_use]
  #[allow(clippy::missing_panics_doc)]
  pub fn trigger_key(&self) -> &Key {
    // SAFETY: Keys vector is verified to be non-empty in
    // `Keybinding::new`.
    self.0.last().unwrap()
  }
}

/// A listener for system-wide keybindings.
#[derive(Debug)]
pub struct KeybindingListener {
  /// A receiver channel for outgoing keybinding events.
  event_rx: mpsc::UnboundedReceiver<KeybindingEvent>,

  /// The keybinding lookup maps used by the keyboard hook.
  ///
  /// Regular keybindings are keyed by the canonical form of their trigger
  /// key (the final key in the keybinding). For example, in the
  /// keybinding `[Key::Cmd, Key::Shift, Key::A]`, `Key::A` is the trigger
  /// key.
  keybinding_maps: Arc<Mutex<KeybindingMaps>>,

  /// Whether the listener is currently enabled.
  enabled: Arc<AtomicBool>,

  /// The underlying keyboard hook used to listen for key events.
  keyboard_hook: platform_impl::KeyboardHook,
}

impl KeybindingListener {
  /// Creates an instance of `KeybindingListener`.
  pub fn new(
    keybindings: &[Keybinding],
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let keybinding_maps =
      Arc::new(Mutex::new(Self::create_keybinding_maps(keybindings)));

    let enabled = Arc::new(AtomicBool::new(true));

    let keyboard_hook = Self::create_keyboard_hook(
      keybinding_maps.clone(),
      enabled.clone(),
      event_tx,
      dispatcher,
    )?;

    Ok(Self {
      event_rx,
      keybinding_maps,
      enabled,
      keyboard_hook,
    })
  }

  /// Returns the next keybinding event from the listener.
  ///
  /// This will block until a keybinding event is available.
  pub async fn next_event(&mut self) -> Option<KeybindingEvent> {
    self.event_rx.recv().await
  }

  /// Updates the keybindings for the keybinding listener.
  ///
  /// # Panics
  ///
  /// If the internal mutex is poisoned.
  pub fn update(&self, keybindings: &[Keybinding]) {
    *self.keybinding_maps.lock().unwrap() =
      Self::create_keybinding_maps(keybindings);
  }

  /// Enables or disables the keybinding listener.
  pub fn enable(&mut self, enabled: bool) {
    self.enabled.store(enabled, Ordering::Relaxed);
  }

  /// Terminates the keybinding listener.
  pub fn terminate(&mut self) -> crate::Result<()> {
    self.keyboard_hook.terminate()
  }

  /// Creates and starts the keyboard hook with the given callback.
  fn create_keyboard_hook(
    keybinding_maps: Arc<Mutex<KeybindingMaps>>,
    enabled: Arc<AtomicBool>,
    event_tx: mpsc::UnboundedSender<KeybindingEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<platform_impl::KeyboardHook> {
    let tap_state = Arc::new(Mutex::new(TapState::default()));

    platform_impl::KeyboardHook::new(
      move |event: platform_impl::KeyEvent| -> bool {
        if !enabled.load(Ordering::Relaxed) {
          return false;
        }

        let Ok(keybinding_maps) = keybinding_maps.lock() else {
          tracing::error!("Failed to acquire lock on keybinding maps.");
          return false;
        };

        // Ignore events we injected ourselves (e.g. disguise keys), to
        // avoid recursion and spurious tap tracking.
        if event.is_injected {
          return false;
        }

        // Canonicalize the pressed key so that side-specific/aliased
        // modifiers (e.g. `VK_LWIN` -> `Key::Win`) match config bindings
        // that use a different variant of the same modifier.
        let canonical = canonical_key(event.key);

        // Handle key releases, which are only relevant for firing "tap"
        // keybindings.
        if !event.is_keypress {
          let Ok(mut tap_state) = tap_state.lock() else {
            return false;
          };

          // Only act if this release completes a clean tap of a bound
          // modifier (i.e. no other key was pressed while it was held).
          let is_clean_tap = tap_state.pending == Some(canonical)
            && !tap_state.dirty
            && keybinding_maps.taps.contains_key(&canonical);

          if tap_state.pending == Some(canonical) {
            tap_state.pending = None;
            tap_state.dirty = false;
          }

          drop(tap_state);

          if !is_clean_tap {
            return false;
          }

          // Suppress the system menu (e.g. the Start menu) by swallowing
          // the real release and re-injecting it behind a disguise key.
          // If injection fails, fall through without swallowing so the
          // modifier doesn't get stuck down.
          let disguised =
            platform_impl::disguise_modifier_release(event.key);

          if let Some(keybinding) = keybinding_maps.taps.get(&canonical) {
            let _ = event_tx.send(KeybindingEvent(keybinding.clone()));
          }

          return disguised;
        }

        // Update "tap" tracking on key press.
        if let Ok(mut tap_state) = tap_state.lock() {
          if keybinding_maps.taps.contains_key(&canonical) {
            // Start (or continue) tracking a tap candidate. Avoid resetting
            // `dirty` on auto-repeat presses of the same key.
            if tap_state.pending != Some(canonical) {
              tap_state.pending = Some(canonical);
              tap_state.dirty = false;
            }
          } else if tap_state.pending.is_some() {
            // Any other key press disqualifies the in-progress tap.
            tap_state.dirty = true;
          }
        }

        // Find keybinding candidates whose trigger key is the pressed key.
        let Some(candidates) = keybinding_maps.regular.get(&canonical) else {
          return false;
        };

        let mut cached_key_states = HashMap::new();

        // Find the matching keybindings based on the pressed keys.
        let matched_keybindings = candidates.iter().filter(|keybinding| {
          keybinding.keys().iter().all(|&key| {
            if canonical_key(key) == canonical {
              return true;
            }

            *cached_key_states
              .entry(key)
              .or_insert_with(|| event.is_key_down(key))
          })
        });

        // Find the longest matching keybinding.
        let Some(longest_keybinding) = matched_keybindings
          .max_by_key(|keybinding| keybinding.keys().len())
        else {
          return false;
        };

        // Reject if any modifier keys not in the keybinding are held.
        let has_extra_modifiers = MODIFIER_GROUPS
          .iter()
          // Filter out modifier groups that have keys in the keybinding.
          .filter(|(_, group_keys)| {
            !group_keys
              .iter()
              .any(|key| longest_keybinding.keys().contains(key))
          })
          // Use the group's "generic" key (e.g. `Key::Shift`) to check if
          // the modifier is held. This avoids lookups for `Key::LShift`
          // and `Key::RShift`.
          .any(|(generic_key, _)| {
            cached_key_states
              .get(generic_key)
              .copied()
              .unwrap_or_else(|| event.is_key_down(*generic_key))
          });

        if has_extra_modifiers {
          return false;
        }

        let _ = event_tx.send(KeybindingEvent(longest_keybinding.clone()));

        true
      },
      dispatcher,
    )
  }

  /// Builds the keybinding lookup maps from configs.
  ///
  /// Single-key bindings consisting only of a modifier (e.g. `lwin`) are
  /// placed into the "tap" map; all other bindings into the "regular" map.
  /// Both are keyed by the canonical form of the relevant key.
  fn create_keybinding_maps(keybindings: &[Keybinding]) -> KeybindingMaps {
    let mut maps = KeybindingMaps::default();

    for keybinding in keybindings {
      let keys = keybinding.keys();

      if keys.len() == 1 && is_modifier(keys[0]) {
        maps.taps.insert(canonical_key(keys[0]), keybinding.clone());
      } else {
        maps
          .regular
          .entry(canonical_key(*keybinding.trigger_key()))
          .or_insert_with(Vec::new)
          .push(keybinding.clone());
      }
    }

    maps
  }
}

impl Drop for KeybindingListener {
  fn drop(&mut self) {
    let _ = self.terminate();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn canonicalizes_modifier_keys() {
    assert_eq!(canonical_key(Key::LWin), Key::Win);
    assert_eq!(canonical_key(Key::RWin), Key::Win);
    assert_eq!(canonical_key(Key::Cmd), Key::Win);
    assert_eq!(canonical_key(Key::Win), Key::Win);
    assert_eq!(canonical_key(Key::LShift), Key::Shift);
    assert_eq!(canonical_key(Key::RCtrl), Key::Ctrl);
    assert_eq!(canonical_key(Key::LAlt), Key::Alt);
    // Non-modifier keys are unchanged.
    assert_eq!(canonical_key(Key::A), Key::A);
  }

  #[test]
  fn detects_modifier_keys() {
    assert!(is_modifier(Key::LWin));
    assert!(is_modifier(Key::Win));
    assert!(is_modifier(Key::Shift));
    assert!(!is_modifier(Key::A));
  }

  #[test]
  fn lone_modifier_binding_is_a_tap() {
    let binding = Keybinding::new(vec![Key::LWin]).unwrap();
    let maps = KeybindingListener::create_keybinding_maps(&[binding.clone()]);

    // A lone modifier binding goes into the tap map, keyed by its
    // canonical form, and not into the regular map.
    assert_eq!(maps.taps.get(&Key::Win), Some(&binding));
    assert!(maps.regular.is_empty());
  }

  #[test]
  fn modifier_combo_binding_is_regular() {
    let binding = Keybinding::new(vec![Key::LWin, Key::A]).unwrap();
    let maps = KeybindingListener::create_keybinding_maps(&[binding.clone()]);

    // A combo binding goes into the regular map, keyed by the canonical
    // form of its (non-modifier) trigger key.
    assert_eq!(maps.regular.get(&Key::A), Some(&vec![binding]));
    assert!(maps.taps.is_empty());
  }

  #[test]
  fn modifier_only_trigger_is_canonicalized() {
    // A binding whose trigger is a side-specific modifier should be keyed
    // by the canonical form so a generic `VK_LWIN -> Key::Win` press
    // matches it.
    let binding = Keybinding::new(vec![Key::Ctrl, Key::LWin]).unwrap();
    let maps = KeybindingListener::create_keybinding_maps(&[binding.clone()]);

    assert_eq!(maps.regular.get(&Key::Win), Some(&vec![binding]));
  }
}
