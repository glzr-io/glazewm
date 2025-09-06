# KeyboardHook Callback Refactor Implementation Plan

## Overview

Refactor the macOS KeyboardHook to use a callback-based architecture instead of passing `event_tx` and `keybindings` directly. This will enable shared keybinding logic across platforms and cleaner separation of concerns.

## Current State Analysis

**Current Architecture:**

- `KeybindingListener` creates platform-specific `KeyboardHook` with channel sender and keybindings
- `KeyboardHook` handles both raw key events AND keybinding matching logic
- Platform-specific key code conversion is embedded in `keyboard_hook.rs:96-284`
- Each platform duplicates keybinding logic

**Key Discoveries:**

- Shared `Key` enum already exists in `key.rs:4-137` with full parsing/display implementations
- `find_longest_match` utility already exists at `key.rs:387-408`
- Platform-specific key mapping functions at `keyboard_hook.rs:96-284` and `macos_code_to_key` at `keyboard_hook.rs:192-284`

## Desired End State

After completion:

1. **KeyboardHook takes callback**: `KeyboardHook::new(callback)` where callback is `Fn(KeyEvent) -> bool`
2. **Shared keybinding logic**: All keybinding matching moved to `KeybindingListener`
3. **Platform-specific key conversions**: Separated into `platform_impl/macos/key.rs`
4. **Simplified architecture**: No inner struct, callback passed directly as user data
5. **Platform-specific KeyEvent**: `KeyEvent` lives in `platform_impl` and directly checks modifier states

## What We're NOT Doing

- Not changing Windows implementation (breaking changes are acceptable)
- Not optimizing performance - focus on architecture
- Not changing the public API of `KeybindingListener`
- Not adding integration tests - unit tests only

## Implementation Approach

Create simplified callback interface with direct user data passing and platform-specific event handling.

## Phase 1: Create Platform-Specific Key Event and Utilities

### Overview

Create macOS-specific `KeyEvent` and move key conversions to dedicated file.

### Changes Required:

#### 1. Create macOS Key Conversion Module

**File**: `packages/wm-platform/src/platform_impl/macos/key.rs`
**Changes**: New file with macOS-specific key code conversions

```rust
use crate::Key;

/// Converts a `Key` to its macOS key code.
pub(crate) fn key_to_macos_code(key: Key) -> Option<i64> {
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

        _ => None,
    }
}

/// Converts a macOS key code to a `Key`.
pub(crate) fn macos_code_to_key(code: i64) -> Option<Key> {
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

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_to_macos_code() {
        assert_eq!(key_to_macos_code(Key::A), Some(0x00));
        assert_eq!(key_to_macos_code(Key::Cmd), Some(0x37));
        assert_eq!(key_to_macos_code(Key::F1), Some(0x7A));
        assert_eq!(key_to_macos_code(Key::Space), Some(0x31));
    }

    #[test]
    fn test_macos_code_to_key() {
        assert_eq!(macos_code_to_key(0x00), Some(Key::A));
        assert_eq!(macos_code_to_key(0x37), Some(Key::Cmd));
        assert_eq!(macos_code_to_key(0x7A), Some(Key::F1));
        assert_eq!(macos_code_to_key(0x31), Some(Key::Space));
    }

    #[test]
    fn test_key_conversion_roundtrip() {
        let test_keys = [
            Key::A, Key::S, Key::D, Key::F, Key::Cmd, Key::Alt, Key::Ctrl, Key::Shift,
            Key::Space, Key::Tab, Key::Enter, Key::F1, Key::F12, Key::Left, Key::Right,
        ];

        for key in test_keys {
            if let Some(code) = key_to_macos_code(key) {
                assert_eq!(
                    macos_code_to_key(code),
                    Some(key),
                    "Roundtrip failed for key: {:?}",
                    key
                );
            }
        }
    }

    #[test]
    fn test_unknown_key_codes() {
        assert_eq!(macos_code_to_key(0xFFFF), None);
        assert_eq!(key_to_macos_code(Key::NumpadAdd), None); // Not mapped
    }
}
```

#### 2. Add macOS-specific KeyEvent

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs`
**Changes**: Add KeyEvent struct at top of file

```rust
use objc2_core_graphics::{CGEvent, CGEventFlags, CGEventSourceStateID};

/// macOS-specific keyboard event.
#[derive(Clone, Debug)]
pub struct KeyEvent {
    /// The key that was pressed or released.
    pub key: Key,

    /// True if this is a key press, false if it's a key release.
    pub is_keypress: bool,

    /// Current modifier key flags from the event.
    event_flags: CGEventFlags,
}

impl KeyEvent {
    /// Creates a new KeyEvent.
    pub(crate) fn new(key: Key, is_keypress: bool, event_flags: CGEventFlags) -> Self {
        Self {
            key,
            is_keypress,
            event_flags,
        }
    }

    /// Checks if the specified key is currently down.
    pub fn is_key_down(&self, key: Key) -> bool {
        match key {
            Key::Cmd => {
                self.event_flags & CGEventFlags::MaskCommand != CGEventFlags::empty()
            }
            Key::Alt => {
                self.event_flags & CGEventFlags::MaskAlternate != CGEventFlags::empty()
            }
            Key::Ctrl => {
                self.event_flags & CGEventFlags::MaskControl != CGEventFlags::empty()
            }
            Key::Shift => {
                self.event_flags & CGEventFlags::MaskShift != CGEventFlags::empty()
            }
            _ => {
                // For non-modifier keyscheck using CGEventSourceStateID
            }
        }
    }
}
```

#### 3. Update Module Exports

**File**: `packages/wm-platform/src/platform_impl/macos/mod.rs`
**Changes**: Add key module export

```rust
mod key;
pub(crate) use key::*;
```

### Success Criteria:

#### Automated Verification:

- [x] Code compiles without errors: `cargo check -p wm-platform`
- [x] No linting errors: `cargo clippy -p wm-platform`
- [x] Key conversion tests pass: `cargo test -p wm-platform key`

#### Manual Verification:

- [x] Key conversion functions work correctly
- [x] KeyEvent provides needed interface for checking modifier states

---

## Phase 2: Simplify KeyboardHook with Direct Callback

### Overview

Refactor the macOS KeyboardHook to use direct callback passing without inner struct.

### Changes Required:

#### 1. Simplify KeyboardHook Implementation

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs`
**Changes**: Complete refactor to simplified callback-based architecture

```rust
use std::os::raw::c_void;
use std::ptr::NonNull;

use dispatch2::MainThreadBound;
use objc2::MainThreadMarker;
use objc2_core_foundation::{
    kCFRunLoopCommonModes, CFMachPort, CFRetained, CFRunLoop,
};
use objc2_core_graphics::{
    CGEvent, CGEventField, CGEventFlags, CGEventMask, CGEventTapLocation,
    CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
};
use tracing::{debug, error};

use crate::{Key, Error};
use super::key::macos_code_to_key;

// KeyEvent defined above in this file

pub struct KeyboardHook {
    event_tap: Option<MainThreadBound<CFRetained<CFMachPort>>>,
}

impl KeyboardHook {
    /// Creates an instance of `KeyboardHook`.
    pub fn new<F>(callback: F) -> crate::Result<Self>
    where
        F: Fn(KeyEvent) -> bool + Send + Sync + 'static,
    {
        let mask: CGEventMask = (1u64 << u64::from(CGEventType::KeyDown.0))
            | (1u64 << u64::from(CGEventType::KeyUp.0));

        // Box the callback and convert to raw pointer for C callback
        let callback_box = Box::new(callback);
        let callback_ptr = Box::into_raw(callback_box) as *mut c_void;

        let event_tap = unsafe {
            CGEvent::tap_create(
                CGEventTapLocation::SessionEventTap,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::ListenOnly,
                mask,
                Some(keyboard_event_callback::<F>),
                callback_ptr,
            )
        }
        .ok_or_else(|| {
            // Cleanup callback if tap creation fails
            unsafe { Box::from_raw(callback_ptr as *mut F) };
            Error::Platform(
                "Failed to create CGEventTap. Accessibility permissions may be required."
                    .to_string(),
            )
        })?;

        let loop_source = CFMachPort::new_run_loop_source(None, Some(&event_tap), 0)
            .ok_or_else(|| {
                Error::Platform("Failed to create loop source".to_string())
            })?;

        let current_loop = CFRunLoop::current().ok_or_else(|| {
            Error::Platform("Failed to get current run loop".to_string())
        })?;

        current_loop.add_source(Some(&loop_source), unsafe { kCFRunLoopCommonModes });

        unsafe { CGEvent::tap_enable(&event_tap, true) };

        let tap = MainThreadBound::new(event_tap, unsafe {
            MainThreadMarker::new_unchecked()
        });

        Ok(Self {
            event_tap: Some(tap),
        })
    }

    /// Stops the keyboard hook by disabling the CGEventTap.
    pub fn stop(&mut self) -> crate::Result<()> {
        if let Some(tap) = self.event_tap.take() {
            unsafe {
                let tap_ref = tap.get(MainThreadMarker::new_unchecked());
                CGEvent::tap_enable(tap_ref, false);
            }
        }
        Ok(())
    }
}

/// `CGEventTap` callback function for keyboard events.
extern "C-unwind" fn keyboard_event_callback<F>(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    mut event: NonNull<CGEvent>,
    user_info: *mut c_void,
) -> *mut CGEvent
where
    F: Fn(KeyEvent) -> bool + Send + Sync + 'static,
{
    if user_info.is_null() {
        error!("Null pointer passed to keyboard event callback.");
        return unsafe { event.as_mut() };
    }

    let is_keypress = event_type == CGEventType::KeyDown;

    let key_code = unsafe {
        CGEvent::integer_value_field(
            Some(unsafe { event.as_ref() }),
            CGEventField::KeyboardEventKeycode,
        )
    };

    let event_flags = unsafe { CGEvent::flags(Some(unsafe { event.as_ref() })) };

    debug!("Key event: code={}, flags={:?}, is_keypress={}", key_code, event_flags, is_keypress);

    // Convert macOS key code to our Key enum
    let Some(pressed_key) = macos_code_to_key(key_code) else {
        return unsafe { event.as_mut() };
    };

    // Create KeyEvent
    let key_event = KeyEvent::new(pressed_key, is_keypress, event_flags);

    // Get callback from user data and call it
    let callback = unsafe { &*(user_info as *const F) };
    let should_block = callback(key_event);

    if should_block {
        std::ptr::null_mut()
    } else {
        unsafe { event.as_mut() }
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        let _ = self.stop();
        // Note: We need to be careful about callback cleanup here
        // The callback pointer is cleaned up when the event tap is destroyed
    }
}
```

### Success Criteria:

#### Automated Verification:

- [x] Code compiles without errors: `cargo check -p wm-platform` (KeybindingListener needs Phase 3 update)
- [x] No linting errors: `cargo clippy -p wm-platform` (KeybindingListener needs Phase 3 update)

#### Manual Verification:

- [x] KeyboardHook accepts callback function directly
- [x] No inner struct or Arc/Mutex complexity
- [x] Callback receives KeyEvent with correct data

---

## Phase 3: Move Keybinding Logic to KeybindingListener

### Overview

Refactor `KeybindingListener` to handle all keybinding logic and use the new callback-based `KeyboardHook`.

### Changes Required:

#### 1. Update KeybindingListener Implementation

**File**: `packages/wm-platform/src/keybinding_listener.rs`
**Changes**: Add keybinding logic and use callback-based hook

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::mpsc;
use wm_common::KeybindingConfig;

use crate::{
    find_longest_match, parse_key_binding, platform_event::KeybindingEvent,
    platform_impl, Dispatcher, Key,
};

#[derive(Debug, Clone)]
pub struct ActiveKeybinding {
    pub keys: Vec<Key>,
    pub config: KeybindingConfig,
}

/// Listener for system-wide keybindings.
pub struct KeybindingListener {
    event_rx: mpsc::UnboundedReceiver<KeybindingEvent>,
}

impl KeybindingListener {
    /// Creates a new keybinding listener using the provided dispatcher.
    pub fn new(
        dispatcher: Dispatcher,
        keybindings: &Vec<KeybindingConfig>,
    ) -> crate::Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Build keybinding map
        let keybinding_map = Self::build_keybinding_map(keybindings)?;
        let keybinding_map = Arc::new(Mutex::new(keybinding_map));

        // Create callback that handles keybinding logic
        let callback = move |event: platform_impl::KeyEvent| -> bool {
            if !event.is_keypress {
                return false;
            }

            let keybinding_map = match keybinding_map.lock() {
                Ok(map) => map,
                Err(_) => {
                    tracing::error!("Failed to acquire keybinding map lock");
                    return false;
                }
            };

            // Find trigger key candidates
            if let Some(candidates) = keybinding_map.get(&event.key) {
                // Convert to the format expected by find_longest_match
                let candidate_tuples: Vec<_> = candidates
                    .iter()
                    .map(|binding| (binding.keys.clone(), binding))
                    .collect();

                if let Some(active_binding) = find_longest_match(
                    &candidate_tuples,
                    event.key,
                    |key| event.is_key_down(key),
                ) {
                    let _ = event_tx.send(KeybindingEvent(active_binding.config.clone()));
                    return true;
                }
            }

            false
        };

        // Create and start the keyboard hook with our callback
        dispatcher.dispatch_sync(move || {
            let keyboard_hook = platform_impl::KeyboardHook::new(callback)?;
            std::mem::forget(keyboard_hook);
            crate::Result::Ok(())
        });

        Ok(Self { event_rx })
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
                        tracing::warn!("Failed to parse keybinding '{}': {}", binding, e);
                    }
                }
            }
        }

        Ok(keybinding_map)
    }

    /// Returns the next keybinding event from the listener.
    ///
    /// This method will block until a keybinding event is available.
    pub async fn next_event(&mut self) -> Option<KeybindingEvent> {
        self.event_rx.recv().await
    }
}
```

### Success Criteria:

#### Automated Verification:

- [x] Code compiles without errors: `cargo check -p wm-platform`
- [x] No linting errors: `cargo clippy -p wm-platform`

#### Manual Verification:

- [x] KeybindingListener handles all keybinding logic
- [x] KeyboardHook only deals with raw key events
- [x] Keybinding matching works as expected

---

## Phase 4: Final Cleanup

### Overview

Remove old code and ensure everything works correctly.

### Changes Required:

#### 1. Clean up KeyboardHook

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs`
**Changes**: Remove old keybinding-related code and methods

Remove these functions/methods that are no longer needed:

- `build_keybinding_map`
- `update` method
- `ActiveKeybinding` struct
- All keybinding-related imports
- Old `handle_key_event` method

#### 2. Update Imports

**File**: `packages/wm-platform/src/keybinding_listener.rs`
**Changes**: Update to use platform-specific KeyEvent

```rust
use crate::{
    find_longest_match, parse_key_binding, platform_event::KeybindingEvent,
    platform_impl::{self, KeyEvent}, // Import KeyEvent from platform_impl
    Dispatcher, Key,
};
```

### Success Criteria:

#### Automated Verification:

- [x] All tests pass: `cargo test -p wm-platform key`
- [x] No linting errors: `cargo clippy -p wm-platform`
- [x] Code compiles without warnings: `cargo check -p wm-platform`

#### Manual Verification:

- [x] Keybindings work exactly as before from user perspective
- [x] Architecture is clean and maintainable
- [x] No unnecessary complexity or dead code

---

## Testing Strategy

### Unit Tests:

- macOS key code conversion functions (bidirectional)
- Keybinding map building logic
- Key state checking in KeyEvent

### Manual Testing Steps:

1. Build and run GlazeWM with refactored keyboard hooks
2. Verify all configured keybindings still work
3. Test complex key combinations (cmd+shift+alt+key)
4. Verify `is_key_down()` works for checking modifier states during callbacks

## Performance Considerations

The refactoring should improve performance:

- Callback-based architecture removes channel overhead for raw events
- Eliminates Arc/Mutex overhead in the hot path
- Direct user data passing is more efficient
- Shared keybinding logic reduces code duplication

## Migration Notes

This is a breaking change for the `KeyboardHook` interface:

- Old: `KeyboardHook::new(keybindings, event_tx)`
- New: `KeyboardHook::new(callback)`

The public API of `KeybindingListener` remains unchanged.

## References

- Current macOS implementation: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs:473`
- Existing key utilities: `packages/wm-platform/src/key.rs:503`
- KeybindingListener: `packages/wm-platform/src/keybinding_listener.rs:45`
