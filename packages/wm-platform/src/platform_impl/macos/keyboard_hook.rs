use std::{
  collections::HashMap,
  os::raw::c_void,
  ptr::NonNull,
  sync::{Arc, Mutex},
};

use dispatch2::MainThreadBound;
use objc2::MainThreadMarker;
use objc2_core_foundation::{
  kCFRunLoopCommonModes, CFMachPort, CFRetained, CFRunLoop,
};
use objc2_core_graphics::{
  CGEvent, CGEventField, CGEventFlags, CGEventMask, CGEventTapLocation,
  CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
};
use tokio::sync::mpsc;
use tracing::{debug, error, warn};
use wm_common::KeybindingConfig;

use crate::{
  find_longest_match, parse_key_binding, Error, Key, KeybindingEvent,
};

#[derive(Debug)]
pub struct KeyboardHook {
  inner: Arc<Mutex<KeyboardHookInner>>,
}

#[derive(Debug)]
struct KeyboardHookInner {
  /// Sender to emit platform events.
  event_tx: mpsc::UnboundedSender<KeybindingEvent>,

  /// CGEventTap handle for keyboard monitoring.
  event_tap: Option<MainThreadBound<CFRetained<CFMachPort>>>,

  /// Active keybindings grouped by trigger key.
  keybindings_by_trigger_key: HashMap<Key, Vec<ActiveKeybinding>>,
}

#[derive(Debug, Clone)]
pub struct ActiveKeybinding {
  pub keys: Vec<Key>,
  pub config: KeybindingConfig,
}

impl KeyboardHook {
  /// Creates an instance of `KeyboardHook`.
  pub fn new(
    keybindings: &[KeybindingConfig],
    event_tx: mpsc::UnboundedSender<KeybindingEvent>,
  ) -> crate::Result<Self> {
    let inner = KeyboardHookInner {
      event_tx,
      event_tap: None,
      keybindings_by_trigger_key: Self::build_keybinding_map(keybindings)?,
    };

    Ok(Self {
      inner: Arc::new(Mutex::new(inner)),
    })
  }

  /// Builds the keybinding map from configs.
  fn build_keybinding_map(
    keybindings: &[KeybindingConfig],
  ) -> crate::Result<HashMap<Key, Vec<ActiveKeybinding>>> {
    let mut keybinding_map = HashMap::new();

    for config in keybindings {
      for binding in &config.bindings {
        match parse_key_binding(binding) {
          Ok(keys) => {
            if let Some(&trigger_key) = keys.last() {
              keybinding_map
                .entry(trigger_key)
                .or_insert_with(Vec::new)
                .push(ActiveKeybinding {
                  keys,
                  config: config.clone(),
                });
            }
          }
          Err(e) => {
            warn!("Failed to parse keybinding '{}': {}", binding, e);
          }
        }
      }
    }

    Ok(keybinding_map)
  }

  /// Converts a `Key` to its macOS key code.
  fn key_to_macos_code(key: Key) -> Option<i64> {
    match key {
      // Letter keys
      Key::A => Some(0x00),
      Key::S => Some(0x01),
      Key::D => Some(0x02),
      Key::F => Some(0x03),
      Key::H => Some(0x04),
      Key::G => Some(0x05),
      Key::Z => Some(0x06),
      Key::X => Some(0x07),
      Key::C => Some(0x08),
      Key::V => Some(0x09),
      Key::B => Some(0x0B),
      Key::Q => Some(0x0C),
      Key::W => Some(0x0D),
      Key::E => Some(0x0E),
      Key::R => Some(0x0F),
      Key::Y => Some(0x10),
      Key::T => Some(0x11),
      Key::O => Some(0x1F),
      Key::U => Some(0x20),
      Key::I => Some(0x22),
      Key::P => Some(0x23),
      Key::L => Some(0x25),
      Key::J => Some(0x26),
      Key::K => Some(0x28),
      Key::N => Some(0x2D),
      Key::M => Some(0x2E),

      // Numbers
      Key::D1 => Some(0x12),
      Key::D2 => Some(0x13),
      Key::D3 => Some(0x14),
      Key::D4 => Some(0x15),
      Key::D6 => Some(0x16),
      Key::D5 => Some(0x17),
      Key::D9 => Some(0x19),
      Key::D7 => Some(0x1A),
      Key::D8 => Some(0x1C),
      Key::D0 => Some(0x1D),

      // Function keys
      Key::F1 => Some(0x7A),
      Key::F2 => Some(0x78),
      Key::F3 => Some(0x63),
      Key::F4 => Some(0x76),
      Key::F5 => Some(0x60),
      Key::F6 => Some(0x61),
      Key::F7 => Some(0x62),
      Key::F8 => Some(0x64),
      Key::F9 => Some(0x65),
      Key::F10 => Some(0x6D),
      Key::F11 => Some(0x67),
      Key::F12 => Some(0x6F),

      // Modifier keys
      Key::Cmd => Some(0x37),
      Key::Alt => Some(0x3A),
      Key::Ctrl => Some(0x3B),
      Key::Shift => Some(0x38),

      // Special keys
      Key::Space => Some(0x31),
      Key::Tab => Some(0x30),
      Key::Enter => Some(0x24),
      Key::Delete => Some(0x33),
      Key::Escape => Some(0x35),

      // Arrow keys
      Key::Left => Some(0x7B),
      Key::Right => Some(0x7C),
      Key::Down => Some(0x7D),
      Key::Up => Some(0x7E),

      // Punctuation
      Key::Equal => Some(0x18),
      Key::Minus => Some(0x1B),
      Key::RightBracket => Some(0x1E),
      Key::LeftBracket => Some(0x21),
      Key::Quote => Some(0x27),
      Key::Semicolon => Some(0x29),
      Key::Backslash => Some(0x2A),
      Key::Comma => Some(0x2B),
      Key::Slash => Some(0x2C),
      Key::Period => Some(0x2F),
      Key::Grave => Some(0x32),

      _ => {
        debug!("Unsupported key for macOS: {:?}", key);
        None
      }
    }
  }

  /// Converts a macOS key code to a `Key`.
  fn macos_code_to_key(code: i64) -> Option<Key> {
    match code {
      // Letter keys
      0x00 => Some(Key::A),
      0x01 => Some(Key::S),
      0x02 => Some(Key::D),
      0x03 => Some(Key::F),
      0x04 => Some(Key::H),
      0x05 => Some(Key::G),
      0x06 => Some(Key::Z),
      0x07 => Some(Key::X),
      0x08 => Some(Key::C),
      0x09 => Some(Key::V),
      0x0B => Some(Key::B),
      0x0C => Some(Key::Q),
      0x0D => Some(Key::W),
      0x0E => Some(Key::E),
      0x0F => Some(Key::R),
      0x10 => Some(Key::Y),
      0x11 => Some(Key::T),
      0x1F => Some(Key::O),
      0x20 => Some(Key::U),
      0x22 => Some(Key::I),
      0x23 => Some(Key::P),
      0x25 => Some(Key::L),
      0x26 => Some(Key::J),
      0x28 => Some(Key::K),
      0x2D => Some(Key::N),
      0x2E => Some(Key::M),

      // Numbers
      0x12 => Some(Key::D1),
      0x13 => Some(Key::D2),
      0x14 => Some(Key::D3),
      0x15 => Some(Key::D4),
      0x16 => Some(Key::D6),
      0x17 => Some(Key::D5),
      0x19 => Some(Key::D9),
      0x1A => Some(Key::D7),
      0x1C => Some(Key::D8),
      0x1D => Some(Key::D0),

      // Function keys
      0x7A => Some(Key::F1),
      0x78 => Some(Key::F2),
      0x63 => Some(Key::F3),
      0x76 => Some(Key::F4),
      0x60 => Some(Key::F5),
      0x61 => Some(Key::F6),
      0x62 => Some(Key::F7),
      0x64 => Some(Key::F8),
      0x65 => Some(Key::F9),
      0x6D => Some(Key::F10),
      0x67 => Some(Key::F11),
      0x6F => Some(Key::F12),

      // Modifier keys
      0x37 => Some(Key::Cmd),
      0x3A => Some(Key::Alt),
      0x3B => Some(Key::Ctrl),
      0x38 => Some(Key::Shift),

      // Special keys
      0x31 => Some(Key::Space),
      0x30 => Some(Key::Tab),
      0x24 => Some(Key::Enter),
      0x33 => Some(Key::Delete),
      0x35 => Some(Key::Escape),

      // Arrow keys
      0x7B => Some(Key::Left),
      0x7C => Some(Key::Right),
      0x7D => Some(Key::Down),
      0x7E => Some(Key::Up),

      // Punctuation
      0x18 => Some(Key::Equal),
      0x1B => Some(Key::Minus),
      0x1E => Some(Key::RightBracket),
      0x21 => Some(Key::LeftBracket),
      0x27 => Some(Key::Quote),
      0x29 => Some(Key::Semicolon),
      0x2A => Some(Key::Backslash),
      0x2B => Some(Key::Comma),
      0x2C => Some(Key::Slash),
      0x2F => Some(Key::Period),
      0x32 => Some(Key::Grave),

      _ => {
        debug!("Unsupported macOS key code: {}", code);
        None
      }
    }
  }

  /// Checks if a modifier key is currently pressed based on event flags.
  fn is_modifier_pressed(key: Key, event_flags: CGEventFlags) -> bool {
    match key {
      Key::Cmd => {
        event_flags & CGEventFlags::MaskCommand != CGEventFlags::empty()
      }
      Key::Alt => {
        event_flags & CGEventFlags::MaskAlternate != CGEventFlags::empty()
      }
      Key::Ctrl => {
        event_flags & CGEventFlags::MaskControl != CGEventFlags::empty()
      }
      Key::Shift => {
        event_flags & CGEventFlags::MaskShift != CGEventFlags::empty()
      }
      _ => false,
    }
  }

  /// Handles a key event and determines if it should be blocked.
  fn handle_key_event(
    inner: &mut KeyboardHookInner,
    event_type: CGEventType,
    event: &CGEvent,
  ) -> bool {
    if event_type != CGEventType::KeyDown {
      return false;
    }

    let key_code = unsafe {
      CGEvent::integer_value_field(
        Some(event),
        CGEventField::KeyboardEventKeycode,
      )
    };

    let event_flags = unsafe { CGEvent::flags(Some(event)) };

    debug!("Key event: code={}, flags={:?}", key_code, event_flags);

    // Convert macOS key code back to our Key enum
    let Some(pressed_key) = Self::macos_code_to_key(key_code) else {
      return false;
    };

    // Find trigger key candidates
    if let Some(candidates) =
      inner.keybindings_by_trigger_key.get(&pressed_key)
    {
      // Convert to the format expected by find_longest_match
      let candidate_tuples: Vec<_> = candidates
        .iter()
        .map(|binding| (binding.keys.clone(), binding))
        .collect();

      if let Some(active_binding) =
        find_longest_match(&candidate_tuples, pressed_key, |key| {
          Self::is_modifier_pressed(key, event_flags)
        })
      {
        let _ = inner
          .event_tx
          .send(KeybindingEvent(active_binding.config.clone()));
        return true;
      }
    }

    false
  }

  /// Starts the keyboard hook by creating and enabling a CGEventTap.
  pub fn start(&mut self) -> crate::Result<()> {
    let mask: CGEventMask = 1u64 << u64::from(CGEventType::KeyDown.0);

    let arc_clone = Arc::clone(&self.inner);
    let arc_ptr = Arc::into_raw(arc_clone) as *mut c_void;

    let event_tap = unsafe {
      CGEvent::tap_create(
        CGEventTapLocation::SessionEventTap,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::ListenOnly,
        mask,
        Some(keyboard_event_callback),
        arc_ptr,
      )
    }
    .ok_or_else(|| {
      Error::Platform(
        "Failed to create CGEventTap. Accessibility permissions may be required."
          .to_string(),
      )
    })?;

    let loop_source =
      CFMachPort::new_run_loop_source(None, Some(&event_tap), 0)
        .ok_or_else(|| {
          Error::Platform("Failed to create loop source".to_string())
        })?;

    let current_loop = CFRunLoop::current().ok_or_else(|| {
      Error::Platform("Failed to get current run loop".to_string())
    })?;

    current_loop
      .add_source(Some(&loop_source), unsafe { kCFRunLoopCommonModes });

    unsafe { CGEvent::tap_enable(&event_tap, true) };

    let tap = MainThreadBound::new(event_tap, unsafe {
      MainThreadMarker::new_unchecked()
    });

    let mut inner = self.inner.lock().map_err(|_| {
      Error::Platform("Failed to acquire mutex".to_string())
    })?;
    inner.event_tap = Some(tap);

    Ok(())
  }

  /// Stops the keyboard hook by disabling the CGEventTap.
  pub fn stop(&mut self) -> crate::Result<()> {
    let mut inner = self.inner.lock().map_err(|_| {
      Error::Platform("Failed to acquire mutex".to_string())
    })?;

    if let Some(tap) = inner.event_tap.take() {
      unsafe {
        let tap_ref = tap.get(MainThreadMarker::new_unchecked());
        CGEvent::tap_enable(tap_ref, false);
      }
    }
    Ok(())
  }

  /// Updates the keybindings for the keyboard hook.
  pub fn update(
    &mut self,
    keybindings: &[KeybindingConfig],
  ) -> crate::Result<()> {
    let mut inner = self.inner.lock().map_err(|_| {
      Error::Platform("Failed to acquire mutex".to_string())
    })?;

    inner.keybindings_by_trigger_key =
      Self::build_keybinding_map(keybindings)?;
    Ok(())
  }
}

/// `CGEventTap` callback function for keyboard events.
extern "C-unwind" fn keyboard_event_callback(
  _proxy: CGEventTapProxy,
  event_type: CGEventType,
  mut event: NonNull<CGEvent>,
  user_info: *mut c_void,
) -> *mut CGEvent {
  if user_info.is_null() {
    error!("Null pointer passed to keyboard event callback.");
    return unsafe { event.as_mut() };
  }

  // Reconstruct the Arc from the raw pointer
  let inner_arc =
    unsafe { Arc::from_raw(user_info.cast::<Mutex<KeyboardHookInner>>()) };

  // Process the event
  let should_block = if let Ok(mut inner) = inner_arc.try_lock() {
    KeyboardHook::handle_key_event(&mut inner, event_type, unsafe {
      event.as_ref()
    })
  } else {
    warn!("Failed to acquire mutex lock in keyboard callback.");
    false
  };

  // Convert back to raw pointer to avoid dropping the Arc.
  let _ = Arc::into_raw(inner_arc);

  if should_block {
    std::ptr::null_mut()
  } else {
    unsafe { event.as_mut() }
  }
}
