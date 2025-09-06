# Keyboard Hook Overhaul Implementation Plan

## Overview

Overhaul and clean up the macOS keyboard hook implementation in wm-platform, moving shared logic into reusable components that can be used by both Windows and macOS implementations.

## Current State Analysis

**macOS Issues (keyboard_hook.rs:404 lines):**

- Debugging `println!` statements scattered throughout (lines 219-262)
- Commented-out code blocks (lines 203-215, 301-313)
- Complex Arc/Mutex + raw pointer callback pattern
- Magic number key codes with no abstraction (lines 104-186)
- Mixed error handling (anyhow vs crate::Error)
- Unsafe code that could be better structured

**Windows Implementation is cleaner:**

- Uses proper Windows VK\_\* constants instead of magic numbers
- Better error handling with proper logging
- Cleaner key state checking with caching
- More sophisticated modifier key handling

**Shared Logic Identified:**

- String-to-keycode mapping functions (nearly identical)

## Desired End State

After this plan is complete:

1. **Shared Components Created:**

   - `Key` enum with standard trait implementations (`FromStr`, `Display`)
   - Shared utility functions for keybinding parsing and matching
   - Clean separation between platform-agnostic logic and platform-specific code

2. **macOS Implementation Cleaned:**

   - Remove all debug `println!` statements
   - Remove commented-out code
   - Simplify callback mechanism
   - Use proper error types consistently
   - Replace magic numbers with `Key` enum

3. **Both Platforms Use Shared Logic:**
   - Windows and macOS both use the same `Key` enum
   - Both use shared keybinding parsing and matching functions
   - Platform-specific code only handles native key code conversion

## What We're NOT Doing

- Not touching Windows implementation functionality (it can break during development)
- Not adding new features, just refactoring existing code
- Not optimizing performance (focus on code quality)

## Implementation Approach

Create shared components first, then refactor macOS to use them, ensuring Windows can eventually adopt them without breaking changes.

## Phase 1: Create Shared Key Components

### Overview

Create the shared `Key` enum and utility functions that both platforms can use.

### Changes Required:

#### 1. Create Shared Key Module

**File**: `packages/wm-platform/src/key.rs`
**Changes**: Create new file with shared key components

````rust
use std::fmt;
use std::str::FromStr;

/// Cross-platform key representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // Letter keys
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Number keys
    D0, D1, D2, D3, D4, D5, D6, D7, D8, D9,

    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19, F20, F21, F22, F23, F24,

    // Modifier keys
    Cmd, Ctrl, Alt, Shift,
    LCmd, RCmd, LCtrl, RCtrl, LAlt, RAlt, LShift, RShift,

    // Special keys
    Space, Tab, Enter, Return, Delete, Escape, Backspace,

    // Arrow keys
    Left, Right, Up, Down,

    // Other keys
    Home, End, PageUp, PageDown, Insert,

    // Punctuation (common ones)
    Semicolon, Quote, Comma, Period, Slash, Backslash,
    LeftBracket, RightBracket, Minus, Equal, Grave,

    // Numpad
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadAdd, NumpadSubtract, NumpadMultiply, NumpadDivide, NumpadDecimal,
}

impl FromStr for Key {
    type Err = KeyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            // Letter keys
            "a" => Ok(Key::A),
            "b" => Ok(Key::B),
            "c" => Ok(Key::C),
            "d" => Ok(Key::D),
            "e" => Ok(Key::E),
            "f" => Ok(Key::F),
            "g" => Ok(Key::G),
            "h" => Ok(Key::H),
            "i" => Ok(Key::I),
            "j" => Ok(Key::J),
            "k" => Ok(Key::K),
            "l" => Ok(Key::L),
            "m" => Ok(Key::M),
            "n" => Ok(Key::N),
            "o" => Ok(Key::O),
            "p" => Ok(Key::P),
            "q" => Ok(Key::Q),
            "r" => Ok(Key::R),
            "s" => Ok(Key::S),
            "t" => Ok(Key::T),
            "u" => Ok(Key::U),
            "v" => Ok(Key::V),
            "w" => Ok(Key::W),
            "x" => Ok(Key::X),
            "y" => Ok(Key::Y),
            "z" => Ok(Key::Z),

            // Numbers
            "0" => Ok(Key::D0),
            "1" => Ok(Key::D1),
            "2" => Ok(Key::D2),
            "3" => Ok(Key::D3),
            "4" => Ok(Key::D4),
            "5" => Ok(Key::D5),
            "6" => Ok(Key::D6),
            "7" => Ok(Key::D7),
            "8" => Ok(Key::D8),
            "9" => Ok(Key::D9),

            // Function keys
            "f1" => Ok(Key::F1),
            "f2" => Ok(Key::F2),
            "f3" => Ok(Key::F3),
            "f4" => Ok(Key::F4),
            "f5" => Ok(Key::F5),
            "f6" => Ok(Key::F6),
            "f7" => Ok(Key::F7),
            "f8" => Ok(Key::F8),
            "f9" => Ok(Key::F9),
            "f10" => Ok(Key::F10),
            "f11" => Ok(Key::F11),
            "f12" => Ok(Key::F12),

            // Modifiers
            "cmd" | "command" => Ok(Key::Cmd),
            "ctrl" | "control" => Ok(Key::Ctrl),
            "alt" | "option" => Ok(Key::Alt),
            "shift" => Ok(Key::Shift),

            // Special keys
            "space" => Ok(Key::Space),
            "tab" => Ok(Key::Tab),
            "enter" | "return" => Ok(Key::Enter),
            "delete" => Ok(Key::Delete),
            "escape" => Ok(Key::Escape),
            "backspace" => Ok(Key::Backspace),

            // Arrow keys
            "left" => Ok(Key::Left),
            "right" => Ok(Key::Right),
            "up" => Ok(Key::Up),
            "down" => Ok(Key::Down),

            // Punctuation
            ";" => Ok(Key::Semicolon),
            "'" => Ok(Key::Quote),
            "," => Ok(Key::Comma),
            "." => Ok(Key::Period),
            "/" => Ok(Key::Slash),
            "\\" => Ok(Key::Backslash),
            "[" => Ok(Key::LeftBracket),
            "]" => Ok(Key::RightBracket),
            "-" => Ok(Key::Minus),
            "=" => Ok(Key::Equal),
            "`" => Ok(Key::Grave),

            _ => Err(KeyParseError::UnknownKey(s.to_string())),
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Key::A => "A", Key::B => "B", Key::C => "C", Key::D => "D", Key::E => "E",
            Key::F => "F", Key::G => "G", Key::H => "H", Key::I => "I", Key::J => "J",
            Key::K => "K", Key::L => "L", Key::M => "M", Key::N => "N", Key::O => "O",
            Key::P => "P", Key::Q => "Q", Key::R => "R", Key::S => "S", Key::T => "T",
            Key::U => "U", Key::V => "V", Key::W => "W", Key::X => "X", Key::Y => "Y",
            Key::Z => "Z",

            Key::D0 => "0", Key::D1 => "1", Key::D2 => "2", Key::D3 => "3", Key::D4 => "4",
            Key::D5 => "5", Key::D6 => "6", Key::D7 => "7", Key::D8 => "8", Key::D9 => "9",

            Key::F1 => "F1", Key::F2 => "F2", Key::F3 => "F3", Key::F4 => "F4",
            Key::F5 => "F5", Key::F6 => "F6", Key::F7 => "F7", Key::F8 => "F8",
            Key::F9 => "F9", Key::F10 => "F10", Key::F11 => "F11", Key::F12 => "F12",

            Key::Cmd => "Cmd", Key::Ctrl => "Ctrl", Key::Alt => "Alt", Key::Shift => "Shift",
            Key::Space => "Space", Key::Tab => "Tab", Key::Enter => "Enter",
            Key::Delete => "Delete", Key::Escape => "Escape",

            Key::Left => "Left", Key::Right => "Right", Key::Up => "Up", Key::Down => "Down",

            Key::Semicolon => ";", Key::Quote => "'", Key::Comma => ",", Key::Period => ".",
            Key::Slash => "/", Key::Backslash => "\\", Key::LeftBracket => "[",
            Key::RightBracket => "]", Key::Minus => "-", Key::Equal => "=", Key::Grave => "`",

            _ => "Unknown",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyParseError {
    #[error("Unknown key: {0}")]
    UnknownKey(String),
}

/// Parses a keybinding string like "cmd+shift+a" into a vector of keys.
pub fn parse_key_binding(binding: &str) -> Result<Keybinding, KeyParseError> {
    binding
        .split('+')
        .map(|key| key.trim().parse())
        .collect()
}

/// Builds a trigger key map from an iterator of (keys, value) pairs.
/// The trigger key is the last key in each key combination.
pub fn build_trigger_map<T>(
    bindings: impl Iterator<Item = (Vec<Key>, T)>,
) -> std::collections::HashMap<Key, Vec<(Vec<Key>, T)>> {
    let mut map = std::collections::HashMap::new();

    for (keys, value) in bindings {
        if let Some(&trigger_key) = keys.last() {
            map.entry(trigger_key)
                .or_insert_with(Vec::new)
                .push((keys, value));
        }
    }

    map
}
```

#### 2. Update Module Exports

**File**: `packages/wm-platform/src/lib.rs`
**Changes**: Add key module export

```rust
pub mod key;
pub use key::{Key, KeyParseError, parse_key_binding, build_trigger_map, find_longest_match};
````

### Success Criteria:

#### Automated Verification:

- [ ] Code compiles without errors: `cargo check -p wm-platform`
- [ ] No linting errors: `cargo clippy -p wm-platform`
- [ ] Key parsing tests pass (if added): `cargo test -p wm-platform key`

#### Manual Verification:

- [ ] Key enum covers all keys used in existing implementations
- [ ] `FromStr` implementation handles all existing key strings
- [ ] Shared functions work with simple test cases

---

## Phase 2: Refactor macOS Implementation

### Overview

Clean up the macOS keyboard hook implementation to use the shared key components and remove all the sloppy AI-generated code.

### Changes Required:

#### 1. Clean Up macOS Keyboard Hook

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs`
**Changes**: Complete rewrite using shared components

```rust
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
use tracing::{debug, error, warn};
use wm_common::KeybindingConfig;

use crate::{
    key::{Key, parse_key_binding, build_trigger_map, find_longest_match},
    platform_event::KeybindingEvent,
    Error
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
    keybindings_by_trigger_key: HashMap<Key, Vec<Keybinding>>,
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
        keybindings: &[Keybinding],
    ) -> crate::Result<HashMap<Key, Vec<(Vec<Key>, Keybinding)>>> {
        let parsed_bindings = keybindings
            .iter()
            .flat_map(|config| {
                config.bindings.iter().filter_map(|binding| {
                    match parse_key_binding(binding) {
                        Ok(keys) => Some((keys, config.clone())),
                        Err(e) => {
                            warn!("Failed to parse keybinding '{}': {}", binding, e);
                            None
                        }
                    }
                })
            });

        Ok(build_trigger_map(parsed_bindings))
    }

    /// Converts a `Key` to its macOS key code.
    fn key_to_macos_code(key: Key) -> Option<i64> {
        match key {
            // Letter keys
            Key::A => Some(0x00), Key::S => Some(0x01), Key::D => Some(0x02),
            Key::F => Some(0x03), Key::H => Some(0x04), Key::G => Some(0x05),
            Key::Z => Some(0x06), Key::X => Some(0x07), Key::C => Some(0x08),
            Key::V => Some(0x09), Key::B => Some(0x0B), Key::Q => Some(0x0C),
            Key::W => Some(0x0D), Key::E => Some(0x0E), Key::R => Some(0x0F),
            Key::Y => Some(0x10), Key::T => Some(0x11),
            Key::O => Some(0x1F), Key::U => Some(0x20), Key::I => Some(0x22),
            Key::P => Some(0x23), Key::L => Some(0x25), Key::J => Some(0x26),
            Key::K => Some(0x28), Key::N => Some(0x2D), Key::M => Some(0x2E),

            // Numbers
            Key::D1 => Some(0x12), Key::D2 => Some(0x13), Key::D3 => Some(0x14),
            Key::D4 => Some(0x15), Key::D6 => Some(0x16), Key::D5 => Some(0x17),
            Key::D9 => Some(0x19), Key::D7 => Some(0x1A), Key::D8 => Some(0x1C),
            Key::D0 => Some(0x1D),

            // Function keys
            Key::F1 => Some(0x7A), Key::F2 => Some(0x78), Key::F3 => Some(0x63),
            Key::F4 => Some(0x76), Key::F5 => Some(0x60), Key::F6 => Some(0x61),
            Key::F7 => Some(0x62), Key::F8 => Some(0x64), Key::F9 => Some(0x65),
            Key::F10 => Some(0x6D), Key::F11 => Some(0x67), Key::F12 => Some(0x6F),

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

    /// Checks if a modifier key is currently pressed based on event flags.
    fn is_modifier_pressed(key: Key, event_flags: CGEventFlags) -> bool {
        match key {
            Key::Cmd => event_flags & CGEventFlags::MaskCommand != CGEventFlags::empty(),
            Key::Alt => event_flags & CGEventFlags::MaskAlternate != CGEventFlags::empty(),
            Key::Ctrl => event_flags & CGEventFlags::MaskControl != CGEventFlags::empty(),
            Key::Shift => event_flags & CGEventFlags::MaskShift != CGEventFlags::empty(),
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
            CGEventGetIntegerValueField(
                Some(event),
                CGEventField::KeyboardEventKeycode,
            )
        };

        let event_flags = unsafe { CGEventGetFlags(Some(event)) };

        debug!("Key event: code={}, flags={:?}", key_code, event_flags);

        // Convert macOS key code back to our Key enum
        let pressed_key = key_code.into();

        // Find trigger key candidates
        if let Some(candidates) = inner.keybindings_by_trigger_key.get(&pressed_key) {
            if let Some(config) = find_longest_match(
                candidates,
                pressed_key,
                |key| Self::is_modifier_pressed(key, event_flags),
            ) {
                let _ = inner.event_tx.send(KeybindingEvent(config.clone()));
                return true;
            }
        }

        false
    }

    /// Starts the keyboard hook by creating and enabling a CGEventTap.
    pub fn start(&mut self) -> crate::Result<()> {
        let mask: CGEventMask = 1u64 << CGEventType::KeyDown.0 as u64;

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

        let loop_source = CFMachPort::new_run_loop_source(None, Some(&event_tap), 0)
            .ok_or_else(|| Error::Platform("Failed to create loop source".to_string()))?;

        let current_loop = CFRunLoop::current()
            .ok_or_else(|| Error::Platform("Failed to get current run loop".to_string()))?;

        current_loop.add_source(Some(&loop_source), unsafe { kCFRunLoopCommonModes });

        unsafe { CGEvent::tap_enable(&event_tap, true) };

        let tap = MainThreadBound::new(event_tap, unsafe {
            MainThreadMarker::new_unchecked()
        });

        let mut inner = self
            .inner
            .lock()
            .map_err(|_| Error::Platform("Failed to acquire mutex".to_string()))?;
        inner.event_tap = Some(tap);

        Ok(())
    }

    /// Stops the keyboard hook by disabling the CGEventTap.
    pub fn stop(&mut self) -> crate::Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| Error::Platform("Failed to acquire mutex".to_string()))?;

        if let Some(tap) = inner.event_tap.take() {
            unsafe {
                let tap_ref = tap.get(MainThreadMarker::new_unchecked());
                CGEvent::tap_enable(tap_ref, false);
            }
        }
        Ok(())
    }

    /// Updates the keybindings for the keyboard hook.
    pub fn update(&mut self, keybindings: &[KeybindingConfig]) -> crate::Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| Error::Platform("Failed to acquire mutex".to_string()))?;

        inner.keybindings_by_trigger_key = Self::build_keybinding_map(keybindings)?;
        Ok(())
    }
}

/// CGEventTap callback function for keyboard events.
extern "C-unwind" fn keyboard_event_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    mut event: NonNull<CGEvent>,
    user_info: *mut c_void,
) -> *mut CGEvent {
    if user_info.is_null() {
        error!("Null pointer passed to keyboard event callback");
        return unsafe { event.as_mut() };
    }

    // Reconstruct the Arc from the raw pointer
    let inner_arc = unsafe { Arc::from_raw(user_info.cast::<Mutex<KeyboardHookInner>>()) };

    // Process the event
    let should_block = if let Ok(mut inner) = inner_arc.try_lock() {
        KeyboardHook::handle_key_event(&mut *inner, event_type, unsafe { event.as_ref() })
    } else {
        warn!("Failed to acquire mutex lock in keyboard callback");
        false
    };

    // Convert back to raw pointer to avoid dropping the Arc
    let _ = Arc::into_raw(inner_arc);

    if should_block {
        std::ptr::null_mut()
    } else {
        unsafe { event.as_mut() }
    }
}
```

### Success Criteria:

#### Automated Verification:

- [ ] macOS code compiles: `cargo check -p wm-platform`
- [ ] No clippy warnings: `cargo clippy -p wm-platform`
- [ ] All debug println statements removed
- [ ] All commented code removed

#### Manual Verification:

- [ ] Keyboard hook still captures key events on macOS
- [ ] Keybindings still trigger properly
- [ ] No accessibility permission errors
- [ ] Clean error handling throughout

---

## Phase 3: Add Reverse Key Code Lookup

### Overview

Complete the macOS implementation by adding efficient reverse lookup from macOS key codes to our `Key` enum.

### Changes Required:

#### 1. Add Reverse Lookup to Key Module

**File**: `packages/wm-platform/src/key.rs`
**Changes**: Add platform-specific conversion traits

```rust
/// Platform-specific key code conversion trait.

// macOS implementation
impl From<i64> for Key {
    fn from(code: i64) -> Option<Self> {
        match code {
            0x00 => Some(Key::A),
            0x01 => Some(Key::S),
            // ... complete reverse mapping
            _ => None,
        }
    }
}

impl From<u16> for Key {
    fn from(code: u16) -> Option<Self> {
        // ...
    }
}
```

### Success Criteria:

#### Automated Verification:

- [ ] Code compiles: `cargo check -p wm-platform`
- [ ] No clippy warnings: `cargo clippy -p wm-platform`
- [ ] Key conversion tests pass: `cargo test key_conversion`

#### Manual Verification:

- [ ] All supported keys convert correctly both ways
- [ ] Unknown key codes are handled gracefully
- [ ] Keybinding matching works with real key events

---

## Phase 4: Final Cleanup and Testing

### Overview

Final polish, testing, and documentation of the refactored code.

### Changes Required:

#### 1. Add Documentation and Tests

**File**: `packages/wm-platform/src/key.rs`
**Changes**: Add comprehensive rustdoc comments and tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_parsing() {
        assert_eq!("a".parse::<Key>().unwrap(), Key::A);
        assert_eq!("cmd".parse::<Key>().unwrap(), Key::Cmd);
        assert_eq!("f1".parse::<Key>().unwrap(), Key::F1);

        assert!("invalid".parse::<Key>().is_err());
    }

    #[test]
    fn test_parse_key_binding() {
        let keys = parse_key_binding("cmd+shift+a").unwrap();
        assert_eq!(keys, vec![Key::Cmd, Key::Shift, Key::A]);
    }

    #[test]
    fn test_build_trigger_map() {
        let bindings = vec![
            (vec![Key::Cmd, Key::A], "action1"),
            (vec![Key::Ctrl, Key::B], "action2"),
        ];

        let map = build_trigger_map(bindings.into_iter());
        assert!(map.contains_key(&Key::A));
        assert!(map.contains_key(&Key::B));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_key_conversion() {
        assert_eq!(Key::A.to_platform_code(), Some(0x00));
        assert_eq!(Key::from_platform_code(0x00), Some(Key::A));

        // Test round-trip conversion
        for key in [Key::A, Key::Cmd, Key::F1, Key::Space] {
            if let Some(code) = key.to_platform_code() {
                assert_eq!(Key::from_platform_code(code), Some(key));
            }
        }
    }
}
```

#### 2. Remove KeybindingConfig Import

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs`
**Changes**: Remove wm_common dependency from platform implementation

Currently the macOS hook imports `KeybindingConfig` directly.

```rust
pub struct KeybindingEvent(pub Vec<Key>);
```

### Success Criteria:

#### Automated Verification:

- [ ] All tests pass: `cargo test -p wm-platform`
- [ ] No clippy warnings: `cargo clippy -p wm-platform -- -D warnings`
- [ ] Documentation builds: `cargo doc -p wm-platform`
- [ ] No unused imports or dead code

#### Manual Verification:

- [ ] Keyboard hooks work exactly as before from user perspective
- [ ] All debug output cleaned up
- [ ] Error messages are helpful and professional
- [ ] Code is well-documented and follows project conventions

---

## Testing Strategy

### Unit Tests:

- Key parsing and conversion functions
- Keybinding map building logic
- Longest match finding algorithm
- Platform-specific key code conversions

### Manual Testing Steps:

1. Build and run GlazeWM with refactored keyboard hooks
2. Verify all configured keybindings still work
3. Test with complex key combinations (cmd+shift+alt+key)
4. Test error handling with invalid keybinding configs
5. Verify no performance regression in key event handling
6. Test accessibility permission flows on macOS

## Performance Considerations

The refactoring should not impact performance:

- Key parsing happens only during configuration, not during event handling
- Trigger key lookup remains O(1) with HashMap
- Longest match finding is still O(n) where n is bindings for that trigger key
- Platform key code conversion adds minimal overhead

## References

- Current macOS implementation: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs:404`
- Current Windows implementation: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs:470`
- Keybinding listener: `packages/wm-platform/src/keybinding_listener.rs:45`
- Platform events: `packages/wm-platform/src/platform_event.rs:39`
