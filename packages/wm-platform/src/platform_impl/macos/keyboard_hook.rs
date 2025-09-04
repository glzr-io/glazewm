use std::{
  collections::HashMap,
  os::raw::{c_int, c_void},
  ptr::NonNull,
  sync::{Arc, Mutex},
};

use dispatch2::MainThreadBound;
use objc2::MainThreadMarker;
use objc2_core_foundation::{
  kCFRunLoopCommonModes, CFMachPort, CFRetained, CFRunLoop,
};
use objc2_core_graphics::{
  CGEvent, CGEventField, CGEventFlags, CGEventGetFlags,
  CGEventGetIntegerValueField, CGEventMask, CGEventSourceFlagsState,
  CGEventSourceStateID, CGEventTapLocation, CGEventTapOptions,
  CGEventTapPlacement, CGEventTapProxy, CGEventType,
};
use tokio::sync::mpsc;
use tracing::event;
use wm_common::KeybindingConfig;

use crate::{platform_event::KeybindingEvent, Error};

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

  /// RunLoop source for event tap.
  run_loop_source: Option<*mut objc2_core_foundation::CFRunLoopSource>,

  /// Active keybindings grouped by trigger key.
  keybindings_by_trigger_key: HashMap<i64, Vec<ActiveKeybinding>>,
}

#[derive(Debug, Clone)]
pub struct ActiveKeybinding {
  pub key_codes: Vec<i64>,
  pub config: KeybindingConfig,
}

impl KeyboardHook {
  /// Creates an instance of `KeyboardHook`.
  pub fn new(
    keybindings: &Vec<KeybindingConfig>,
    event_tx: mpsc::UnboundedSender<KeybindingEvent>,
  ) -> crate::Result<Self> {
    let inner = KeyboardHookInner {
      event_tx,
      event_tap: None,
      run_loop_source: None,
      keybindings_by_trigger_key: Self::keybindings_by_trigger_key(
        keybindings,
      ),
    };

    Ok(Self {
      inner: Arc::new(Mutex::new(inner)),
    })
  }

  /// Groups keybindings by their trigger key (last key in the
  /// combination).
  fn keybindings_by_trigger_key(
    keybindings: &Vec<KeybindingConfig>,
  ) -> HashMap<i64, Vec<ActiveKeybinding>> {
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
  fn string_to_key_code(key: &str) -> Option<i64> {
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
  fn handle_key_event(
    inner: &mut KeyboardHookInner,
    event_type: CGEventType,
    event: &CGEvent,
  ) -> bool {
    let key_code = unsafe {
      CGEventGetIntegerValueField(
        Some(event),
        CGEventField::KeyboardEventKeycode,
      )
    };

    let event_flags = unsafe { CGEventGetFlags(Some(event)) };
    // let key_code = unsafe {
    //   CGEventGetIntegerValueField(
    //     Some(&event.as_ref()),
    //     CGEventField::KeyboardEventKeycode,
    //   ) as i64
    // };

    // let event_flags = unsafe {
    //   CGEventGetIntegerValueField(
    //     Some(&event.as_ref()),
    //     CGEventField::EventFlags,
    //   )
    // };

    tracing::info!("Key code: {:?} {:?}", key_code, event_type);

    println!("gets here 1");
    match inner.keybindings_by_trigger_key.get(&key_code) {
      None => false,
      Some(bindings) => {
        println!("gets here 2");
        println!("bindings: {:?}", inner.keybindings_by_trigger_key);

        println!("gets here 3");
        let matched_bindings: Vec<_> = bindings
          .iter()
          .filter(|binding| {
            binding.key_codes.iter().all(|&code| {
              if code == key_code {
                return true;
              }
              Self::is_modifier_pressed_from_flags(code, event_flags)
            })
          })
          .collect();

        println!("gets here 4");
        println!("matched_bindings: {:?}", matched_bindings);
        if let Some(longest_binding) =
          matched_bindings.iter().max_by_key(|b| b.key_codes.len())
        {
          println!("gets here 5");
          let _ = inner
            .event_tx
            .send(KeybindingEvent(longest_binding.config.clone()));
          return true;
        }

        println!("gets here 6");
        false
      }
    }
  }

  /// Checks if a modifier key is currently pressed based on event flags.
  fn is_modifier_pressed_from_flags(
    key_code: i64,
    event_flags: CGEventFlags,
  ) -> bool {
    println!("is_modifier_pressed_from_flags: {:?}", key_code);
    // let flags = CGEventFlags::from_bits_retain(event_flags);
    let res = match key_code {
      0x37 => {
        event_flags & CGEventFlags::MaskCommand != CGEventFlags::empty()
      } /* cmd */
      0x3A => {
        event_flags & CGEventFlags::MaskAlternate != CGEventFlags::empty()
      } /* option/alt */
      0x3B => {
        event_flags & CGEventFlags::MaskControl != CGEventFlags::empty()
      } /* control */
      0x38 => {
        event_flags & CGEventFlags::MaskShift != CGEventFlags::empty()
      } /* shift */
      _ => false,
    };
    println!("is_modifier_pressed_from_flags: {:?}", res);
    res
  }

  /// Starts the keyboard hook by creating and enabling a CGEventTap.
  pub fn start(&mut self) -> crate::Result<()> {
    let mask: CGEventMask = 1u64 << CGEventType::KeyDown.0 as u64;

    // Clone the Arc to increment reference count for the callback
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
      }.ok_or(Error::Platform("Failed to create CGEventTap. Accessibility permissions may be required.".to_string()))?;

    // let event_tap = CGEvent::tap_create(
    //   kCGSessionEventTap,
    //   kCGHeadInsertEventTap,
    //   kCGEventTapOptionDefault,
    //   event_mask,
    //   keyboard_event_callback,
    //   self as *const Self as *mut c_void,
    // );

    println!("======================Event tap: {:?}", event_tap);
    // if event_tap.is_null() {
    //   return Err(Error::Platform("Failed to create CGEventTap.
    // Accessibility permissions may be required.".to_string())); }

    let loop_ = CFMachPort::new_run_loop_source(None, Some(&event_tap), 0)
      .ok_or(anyhow::anyhow!("Failed to create loop source"))?;

    let current_loop = CFRunLoop::current().unwrap();
    current_loop
      .add_source(Some(&loop_), unsafe { kCFRunLoopCommonModes });

    unsafe { CGEvent::tap_enable(&event_tap, true) };

    let tap = MainThreadBound::new(event_tap, unsafe {
      MainThreadMarker::new_unchecked()
    });

    let mut inner = self.inner.lock().unwrap();
    inner.event_tap = Some(tap);

    Ok(())
  }

  /// Stops the keyboard hook by disabling the CGEventTap.
  pub fn stop(&mut self) -> crate::Result<()> {
    let mut inner = self.inner.lock().unwrap();
    if let Some(tap) = inner.event_tap.take() {
      unsafe {
        // Disable the event tap
        let tap_ref = tap.get(MainThreadMarker::new_unchecked());
        CGEvent::tap_enable(tap_ref, false);
      }
    }
    Ok(())
  }

  /// Updates the keybindings for the keyboard hook.
  pub fn update(
    &mut self,
    keybindings: &Vec<KeybindingConfig>,
  ) -> crate::Result<()> {
    let mut inner = self.inner.lock().unwrap();
    inner.keybindings_by_trigger_key =
      Self::keybindings_by_trigger_key(keybindings);
    Ok(())
  }
}

/// CGEventTap callback function for keyboard events.
///
/// This is called by macOS whenever a keyboard event occurs that matches
/// our event mask. It processes the event and determines if it should be
/// blocked (not forwarded to other applications).
extern "C-unwind" fn keyboard_event_callback(
  _proxy: CGEventTapProxy,
  event_type: CGEventType,
  mut event: NonNull<CGEvent>,
  user_info: *mut c_void,
) -> *mut CGEvent {
  tracing::info!("Key code: {:?} {:?}", event_type, event);
  // Only process key down events
  if event_type != CGEventType::KeyDown {
    return event.as_ptr();
  }

  if user_info.is_null() {
    tracing::error!("Null pointer passed to Event Handler.");
    return unsafe { event.as_mut() };
  }

  // Reconstruct the Arc from the raw pointer
  let inner_arc =
    unsafe { Arc::from_raw(user_info.cast::<Mutex<KeyboardHookInner>>()) };

  // Access the inner data through the mutex
  let intercept = if let Ok(mut inner) = inner_arc.try_lock() {
    KeyboardHook::handle_key_event(&mut *inner, event_type, unsafe {
      event.as_ref()
    })
  } else {
    tracing::warn!("Failed to acquire mutex lock in keyboard callback");
    false
  };

  // Convert back to raw pointer to avoid dropping the Arc
  let _ = Arc::into_raw(inner_arc);

  if intercept {
    std::ptr::null_mut()
  } else {
    unsafe { event.as_mut() }
  }
}
