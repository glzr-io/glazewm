use std::{
  collections::HashMap,
  os::raw::{c_int, c_void},
  sync::{Arc, Mutex},
};

use objc2_core_foundation::CFMachPort;
use objc2_core_graphics::{CGEvent, CGEventType};
use tokio::sync::mpsc;
use wm_common::KeybindingConfig;

use crate::{platform_event::KeybindingEvent, Error};

// FFI definitions for missing CGEventTap functionality
type CGEventRef = *mut CGEvent;

type CGEventTapCallBack = unsafe extern "C" fn(
  proxy: CGEventTapProxy,
  event_type: CGEventType,
  event: CGEventRef,
  user_info: *mut c_void,
) -> CGEventRef;

type CGEventTapProxy = *mut c_void;

type CGEventTapLocation = u32;
const kCGSessionEventTap: CGEventTapLocation = 0;

type CGEventTapPlacement = u32;
const kCGHeadInsertEventTap: CGEventTapPlacement = 0;

type CGEventTapOptions = u32;
const kCGEventTapOptionDefault: CGEventTapOptions = 0;

const kCGEventKeyDown: CGEventType = CGEventType(10);

type CGEventField = u32;
const kCGKeyboardEventKeycode: CGEventField = 9;

type CGEventSourceStateID = c_int;
const kCGEventSourceStatePrivate: CGEventSourceStateID = -1;

type CGEventFlags = u64;
const kCGEventFlagMaskCommand: CGEventFlags = 0x100000;
const kCGEventFlagMaskAlternate: CGEventFlags = 0x80000;
const kCGEventFlagMaskControl: CGEventFlags = 0x40000;
const kCGEventFlagMaskShift: CGEventFlags = 0x20000;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
  fn CGEventTapCreate(
    tap: CGEventTapLocation,
    place: CGEventTapPlacement,
    options: CGEventTapOptions,
    events_of_interest: u64,
    callback: CGEventTapCallBack,
    user_info: *mut c_void,
  ) -> *mut CFMachPort;

  fn CGEventTapEnable(tap: *mut CFMachPort, enable: bool);

  fn CGEventGetIntegerValueField(
    event: CGEventRef,
    field: CGEventField,
  ) -> i64;

  fn CGEventSourceFlagsState(
    state_id: CGEventSourceStateID,
  ) -> CGEventFlags;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
  fn CFRelease(cf: *const c_void);
}

#[derive(Debug)]
pub struct KeyboardHook {
  /// Sender to emit platform events.
  event_tx: mpsc::UnboundedSender<KeybindingEvent>,

  /// CGEventTap handle for keyboard monitoring.
  event_tap: Arc<Mutex<Option<*mut CFMachPort>>>,

  /// RunLoop source for event tap.
  run_loop_source:
    Arc<Mutex<Option<*mut objc2_core_foundation::CFRunLoopSource>>>,

  /// Active keybindings grouped by trigger key.
  keybindings_by_trigger_key:
    Arc<Mutex<HashMap<u16, Vec<ActiveKeybinding>>>>,
}

#[derive(Debug, Clone)]
pub struct ActiveKeybinding {
  pub key_codes: Vec<u16>,
  pub config: KeybindingConfig,
}

impl KeyboardHook {
  /// Creates an instance of `KeyboardHook`.
  pub fn new(
    keybindings: &Vec<KeybindingConfig>,
    event_tx: mpsc::UnboundedSender<KeybindingEvent>,
  ) -> crate::Result<Arc<Self>> {
    let keyboard_hook = Arc::new(Self {
      event_tx,
      event_tap: Arc::new(Mutex::new(None)),
      run_loop_source: Arc::new(Mutex::new(None)),
      keybindings_by_trigger_key: Arc::new(Mutex::new(
        Self::keybindings_by_trigger_key(keybindings),
      )),
    });

    Ok(keyboard_hook)
  }

  /// Groups keybindings by their trigger key (last key in the
  /// combination).
  fn keybindings_by_trigger_key(
    keybindings: &Vec<KeybindingConfig>,
  ) -> HashMap<u16, Vec<ActiveKeybinding>> {
    let mut keybinding_map = HashMap::new();

    for keybinding in keybindings {
      for binding in &keybinding.bindings {
        let key_codes = binding
          .split('+')
          .filter_map(|key| Self::string_to_key_code(key.trim()))
          .collect::<Vec<_>>();

        if let Some(&trigger_key) = key_codes.last() {
          keybinding_map
            .entry(trigger_key)
            .or_insert_with(Vec::new)
            .push(ActiveKeybinding {
              key_codes,
              config: keybinding.clone(),
            });
        }
      }
    }

    keybinding_map
  }

  /// Converts a string key name to its corresponding macOS key code.
  fn string_to_key_code(key: &str) -> Option<u16> {
    match key.to_lowercase().as_str() {
      // Letter keys
      "a" => Some(0x00),
      "s" => Some(0x01),
      "d" => Some(0x02),
      "f" => Some(0x03),
      "h" => Some(0x04),
      "g" => Some(0x05),
      "z" => Some(0x06),
      "x" => Some(0x07),
      "c" => Some(0x08),
      "v" => Some(0x09),
      "b" => Some(0x0B),
      "q" => Some(0x0C),
      "w" => Some(0x0D),
      "e" => Some(0x0E),
      "r" => Some(0x0F),
      "y" => Some(0x10),
      "t" => Some(0x11),
      "1" => Some(0x12),
      "2" => Some(0x13),
      "3" => Some(0x14),
      "4" => Some(0x15),
      "6" => Some(0x16),
      "5" => Some(0x17),
      "=" => Some(0x18),
      "9" => Some(0x19),
      "7" => Some(0x1A),
      "-" => Some(0x1B),
      "8" => Some(0x1C),
      "0" => Some(0x1D),
      "]" => Some(0x1E),
      "o" => Some(0x1F),
      "u" => Some(0x20),
      "[" => Some(0x21),
      "i" => Some(0x22),
      "p" => Some(0x23),
      "l" => Some(0x25),
      "j" => Some(0x26),
      "'" => Some(0x27),
      "k" => Some(0x28),
      ";" => Some(0x29),
      "\\" => Some(0x2A),
      "," => Some(0x2B),
      "/" => Some(0x2C),
      "n" => Some(0x2D),
      "m" => Some(0x2E),
      "." => Some(0x2F),
      "`" => Some(0x32),

      // Modifier keys - use the left variants by default
      "cmd" | "command" => Some(0x37),
      "option" | "alt" => Some(0x3A),
      "control" | "ctrl" => Some(0x3B),
      "shift" => Some(0x38),

      // Function keys
      "f1" => Some(0x7A),
      "f2" => Some(0x78),
      "f3" => Some(0x63),
      "f4" => Some(0x76),
      "f5" => Some(0x60),
      "f6" => Some(0x61),
      "f7" => Some(0x62),
      "f8" => Some(0x64),
      "f9" => Some(0x65),
      "f10" => Some(0x6D),
      "f11" => Some(0x67),
      "f12" => Some(0x6F),

      // Special keys
      "space" => Some(0x31),
      "tab" => Some(0x30),
      "return" | "enter" => Some(0x24),
      "delete" => Some(0x33),
      "escape" => Some(0x35),

      // Arrow keys
      "left" => Some(0x7B),
      "right" => Some(0x7C),
      "down" => Some(0x7D),
      "up" => Some(0x7E),

      _ => None,
    }
  }

  /// Handles a key event and determines if it should be blocked.
  fn handle_key_event(&self, key_code: u16) -> bool {
    let keybindings = self.keybindings_by_trigger_key.lock().unwrap();

    match keybindings.get(&key_code) {
      None => false,
      Some(bindings) => {
        let matched_bindings = bindings.iter().filter(|binding| {
          binding.key_codes.iter().all(|&code| {
            if code == key_code {
              return true;
            }
            Self::is_modifier_pressed(code)
          })
        });

        if let Some(longest_binding) =
          matched_bindings.max_by_key(|b| b.key_codes.len())
        {
          let _ = self
            .event_tx
            .send(KeybindingEvent(longest_binding.config.clone()));
          return true;
        }

        false
      }
    }
  }

  /// Checks if a modifier key is currently pressed.
  fn is_modifier_pressed(key_code: u16) -> bool {
    unsafe {
      let flags = CGEventSourceFlagsState(kCGEventSourceStatePrivate);
      match key_code {
        0x37 => flags & kCGEventFlagMaskCommand != 0, // cmd
        0x3A => flags & kCGEventFlagMaskAlternate != 0, // option/alt
        0x3B => flags & kCGEventFlagMaskControl != 0, // control
        0x38 => flags & kCGEventFlagMaskShift != 0,   // shift
        _ => false,
      }
    }
  }

  /// Starts the keyboard hook by creating and enabling a CGEventTap.
  pub fn start(&self) -> crate::Result<()> {
    unsafe {
      let event_mask = 1u64 << kCGEventKeyDown.0;
      let event_tap = CGEventTapCreate(
        kCGSessionEventTap,
        kCGHeadInsertEventTap,
        kCGEventTapOptionDefault,
        event_mask,
        keyboard_event_callback,
        self as *const Self as *mut c_void,
      );

      if event_tap.is_null() {
        return Err(Error::Platform("Failed to create CGEventTap. Accessibility permissions may be required.".to_string()));
      }

      CGEventTapEnable(event_tap, true);
      *self.event_tap.lock().unwrap() = Some(event_tap);
    }

    Ok(())
  }

  /// Stops the keyboard hook by disabling the CGEventTap.
  pub fn stop(&self) -> crate::Result<()> {
    let mut event_tap = self.event_tap.lock().unwrap();

    if let Some(tap) = *event_tap {
      unsafe {
        CGEventTapEnable(tap, false);
        CFRelease(tap as *const c_void);
      }
    }

    *event_tap = None;

    Ok(())
  }

  /// Updates the keybindings for the keyboard hook.
  pub fn update(
    &self,
    keybindings: &Vec<KeybindingConfig>,
  ) -> crate::Result<()> {
    *self.keybindings_by_trigger_key.lock().unwrap() =
      Self::keybindings_by_trigger_key(keybindings);
    Ok(())
  }
}

/// CGEventTap callback function for keyboard events.
///
/// This is called by macOS whenever a keyboard event occurs that matches
/// our event mask. It processes the event and determines if it should be
/// blocked (not forwarded to other applications).
unsafe extern "C" fn keyboard_event_callback(
  _proxy: CGEventTapProxy,
  event_type: CGEventType,
  event: CGEventRef,
  user_info: *mut c_void,
) -> CGEventRef {
  // Only process key down events
  if event_type != kCGEventKeyDown {
    return event;
  }

  // Get the keyboard hook instance from the user_info pointer
  let hook = &*(user_info as *const KeyboardHook);

  // Extract the key code from the event
  let key_code =
    CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode) as u16;

  // Check if this key event should trigger a keybinding
  let should_block = hook.handle_key_event(key_code);

  // If we should block the event, return null to prevent it from
  // being forwarded to other applications
  if should_block {
    return std::ptr::null_mut();
  }

  // Otherwise, let the event pass through normally
  event
}
