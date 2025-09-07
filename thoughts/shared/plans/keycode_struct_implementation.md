# KeyCode Struct Implementation Plan

## Overview

Create a new `KeyCode` struct similar to `WindowId` and `MonitorId` that wraps platform-specific key code types, replacing raw `u16` (Windows) and `i64` (macOS) usage throughout the keyboard handling system. Add support for unknown key codes via an `Unknown(KeyCode)` variant in the `Key` enum.

## Current State Analysis

**Existing ID Pattern Reference:**

- `WindowId` and `DisplayId` in `packages/wm-platform/src/` use the cfg-based wrapping pattern
- Both implement standard traits: `Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash`
- Platform-specific types: Windows (`isize`), macOS (`u32`)

**Current Key Code Usage:**

- **Windows**: Uses `u16` VK codes in `KeyEvent::vk_code` field (`packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs:50`)
- **macOS**: Uses `i64` key codes extracted from `CGEvent` but not stored in `KeyEvent`
- Conversion functions: `vk_code_to_key()`, `key_to_vk_code()` (Windows), `key_to_macos_code()`, `macos_code_to_key()` (macOS)

**Key Enum Structure:**

- 78+ variants in `packages/wm-platform/src/models/key.rs`
- Comprehensive `Display` and `FromStr` implementations
- `KeyParseError::UnknownKey(String)` for invalid keys

## Desired End State

After this implementation:

1. New `KeyCode` struct that wraps `u16` (Windows) and `i64` (macOS)
2. Replace all raw key code usage with `KeyCode` instances
3. Convert standalone functions to `From`/`TryFrom` trait implementations
4. Add `Key::Unknown(KeyCode)` variant for unmapped key codes
5. Add Windows keyboard layout validation logic

### Key Discoveries:

- Windows hook stores VK code in `KeyEvent::vk_code: u16` at `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs:50`
- macOS extracts key codes via `CGEvent::integer_value_field()` at `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs:169-174`
- Conversion functions follow Option pattern for unmapped keys
- ID structs use `pub(crate)` fields for platform-specific access

## What We're NOT Doing

- Not changing the external API of `KeybindingListener`
- Not modifying existing `Key` enum variants (only adding `Unknown`)
- Not changing the keyboard hook callback signatures
- Not altering the cross-platform `Display`/`FromStr` implementations for existing keys
- Not modifying the `KeyEvent` struct interfaces

## Implementation Approach

Incremental implementation that maintains backward compatibility while systematically replacing raw key codes with the new `KeyCode` struct.

## Phase 1: Create KeyCode Struct Foundation

### Overview

Create the `KeyCode` struct following the `WindowId`/`DisplayId` pattern and place it in the models module alongside the existing `Key` enum.

### Changes Required:

#### 1. KeyCode Struct Definition

**File**: `packages/wm-platform/src/models/key.rs`
**Changes**: Add new `KeyCode` struct and related types

```rust
/// Platform-specific keyboard key code.
///
/// Represents the raw key code from the underlying platform's keyboard API.
/// Use this when you need to work with platform-specific key codes directly.
///
/// # Platform-specific
///
/// - **Windows**: `u16` (Virtual Key code from Windows API)
/// - **macOS**: `i64` (Key code from Core Graphics Events)
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyCode(
  #[cfg(target_os = "windows")] pub(crate) u16,
  #[cfg(target_os = "macos")] pub(crate) i64,
);

```

#### 2. Add Unknown Variant to Key Enum

**File**: `packages/wm-platform/src/models/key.rs`
**Changes**: Add `Unknown(KeyCode)` variant to the `Key` enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
  // ... existing variants (A, B, C, etc.)

  // Unknown key codes that couldn't be mapped to a known key
  Unknown(KeyCode),
}
```

#### 3. Update Key Display Implementation

**File**: `packages/wm-platform/src/models/key.rs`
**Changes**: Add display case for `Unknown` variant

```rust
impl fmt::Display for Key {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match self {
      // ... existing cases
      Key::Unknown(keycode) => return write!(f, "Unknown({})", keycode),
    };

    write!(f, "{s}")
  }
}
```

#### 4. Update Key FromStr Implementation

**File**: `packages/wm-platform/src/models/key.rs`
**Changes**: Update parsing to handle `Unknown` keys (will not parse from strings)

```rust
impl FromStr for Key {
  type Err = KeyParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      // ... existing cases

      // Unknown keys cannot be parsed from strings
      _ => Err(KeyParseError::UnknownKey(s.to_string())),
    }
  }
}
```

### Success Criteria:

#### Automated Verification:

- [x] Code compiles successfully: `cargo check -p wm-platform`
- [x] Unit tests pass: `cargo test -p wm-platform key`
- [x] No clippy warnings: `cargo clippy -p wm-platform`

#### Manual Verification:

- [x] `KeyCode` struct follows same patterns as `WindowId`/`DisplayId`
- [x] `Key::Unknown` variant displays correctly
- [x] Raw key code access works properly
- [x] All traits are properly implemented

---

## Phase 2: Implement From/TryFrom Traits

### Overview

Replace the existing standalone conversion functions with trait-based implementations on `KeyCode`, providing a cleaner and more idiomatic API.

### Changes Required:

#### 1. Windows From/TryFrom Implementations

**File**: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs`
**Changes**: Replace functions with trait implementations

```rust
impl From<u16> for KeyCode {
  fn from(vk_code: u16) -> Self {
    Self::from_raw(vk_code)
  }
}

impl From<KeyCode> for u16 {
  fn from(keycode: KeyCode) -> Self {
    keycode.raw()
  }
}

impl TryFrom<KeyCode> for Key {
  type Error = KeyConversionError;

  fn try_from(keycode: KeyCode) -> Result<Self, Self::Error> {
    vk_code_to_key(keycode.raw()).ok_or_else(|| {
      KeyConversionError::UnknownKeyCode(keycode)
    })
  }
}

impl TryFrom<Key> for KeyCode {
  type Error = KeyConversionError;

  fn try_from(key: Key) -> Result<Self, Self::Error> {
    match key {
      Key::Unknown(keycode) => Ok(keycode),
      _ => key_to_vk_code(key)
        .map(Self::from_raw)
        .ok_or_else(|| KeyConversionError::UnsupportedKey(key)),
    }
  }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyConversionError {
  #[error("Unknown key code: {0}")]
  UnknownKeyCode(KeyCode),
  #[error("Unsupported key: {0}")]
  UnsupportedKey(Key),
}
```

#### 2. macOS From/TryFrom Implementations

**File**: `packages/wm-platform/src/platform_impl/macos/key.rs`
**Changes**: Add trait implementations for macOS

```rust
impl From<i64> for KeyCode {
  fn from(code: i64) -> Self {
    Self::from_raw(code)
  }
}

impl From<KeyCode> for i64 {
  fn from(keycode: KeyCode) -> Self {
    keycode.raw()
  }
}

impl TryFrom<KeyCode> for Key {
  type Error = KeyConversionError;

  fn try_from(keycode: KeyCode) -> Result<Self, Self::Error> {
    macos_code_to_key(keycode.raw()).ok_or_else(|| {
      KeyConversionError::UnknownKeyCode(keycode)
    })
  }
}

impl TryFrom<Key> for KeyCode {
  type Error = KeyConversionError;

  fn try_from(key: Key) -> Result<Self, Self::Error> {
    match key {
      Key::Unknown(keycode) => Ok(keycode),
      _ => key_to_macos_code(key)
        .map(Self::from_raw)
        .ok_or_else(|| KeyConversionError::UnsupportedKey(key)),
    }
  }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyConversionError {
  #[error("Unknown key code: {0}")]
  UnknownKeyCode(KeyCode),
  #[error("Unsupported key: {0}")]
  UnsupportedKey(Key),
}
```

#### 3. Update Module Exports

**File**: `packages/wm-platform/src/models/mod.rs`
**Changes**: Export the new `KeyCode` type

```rust
pub use key::{Key, KeyParseError, KeyCode};
```

### Success Criteria:

#### Automated Verification:

- [x] Code compiles: `cargo check -p wm-platform`
- [x] Unit tests pass: `cargo test -p wm-platform key`
- [x] No clippy warnings: `cargo clippy -p wm-platform`

#### Manual Verification:

- [x] `KeyCode::try_from(Key)` works for all existing keys
- [x] `Key::try_from(KeyCode)` works for all mapped key codes
- [x] Error handling works correctly for unmapped codes
- [x] Conversion is bidirectional where possible

---

## Phase 3: Replace Raw Key Code Usage

### Overview

Systematically replace raw `u16`/`i64` usage with `KeyCode` throughout the keyboard handling system.

### Changes Required:

#### 1. Update Windows KeyEvent

**File**: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs`
**Changes**: Replace `u16` VK code with `KeyCode`

```rust
/// Windows-specific keyboard event.
#[derive(Clone, Debug)]
pub struct KeyEvent {
  /// Whether the event is for a key press or release.
  pub is_keypress: bool,

  /// The key that was pressed or released.
  pub key: Key,

  /// Key code of the pressed key.
  keycode: KeyCode,
}

impl KeyEvent {
  /// Creates an instance of `KeyEvent`.
  pub(crate) fn new(key: Key, is_keypress: bool, keycode: KeyCode) -> Self {
    Self {
      is_keypress,
      key,
      keycode,
    }
  }

  /// Gets the raw key code for this event.
  pub fn keycode(&self) -> KeyCode {
    self.keycode
  }

  /// Gets whether the specified key is currently pressed.
  pub fn is_key_down(&self, key: Key) -> bool {
    match key {
      Key::Cmd | Key::Win => {
        Self::is_key_down_keycode(KeyCode::from_raw(VK_LWIN.0))
          || Self::is_key_down_keycode(KeyCode::from_raw(VK_RWIN.0))
      }
      // ... similar for other modifiers
      _ => {
        if let Ok(keycode) = KeyCode::try_from(key) {
          Self::is_key_down_keycode(keycode)
        } else {
          false
        }
      }
    }
  }

  fn is_key_down_keycode(keycode: KeyCode) -> bool {
    unsafe { (GetKeyState(keycode.raw().into()) & 0x80) == 0x80 }
  }
}
```

#### 2. Update Windows Hook Procedure

**File**: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs`
**Changes**: Use `KeyCode` in hook procedure

```rust
extern "system" fn hook_proc(
  code: i32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  // ... existing code ...

  let vk_code = input.vkCode as u16;
  let keycode = KeyCode::from_raw(vk_code);

  // Try to convert to known key first, fall back to Unknown
  let key = Key::try_from(keycode).unwrap_or(Key::Unknown(keycode));

  let key_event = KeyEvent::new(key, is_keydown, keycode);

  // ... rest of existing code ...
}
```

#### 3. Update macOS Hook Implementation

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs`
**Changes**: Use `KeyCode` in event callback

```rust
extern "C-unwind" fn keyboard_event_callback<F>(
  _proxy: CGEventTapProxy,
  event_type: CGEventType,
  mut event: NonNull<CGEvent>,
  user_info: *mut c_void,
) -> *mut CGEvent
where
  F: Fn(KeyEvent) -> bool + Send + Sync + 'static,
{
  // ... existing setup code ...

  let key_code = unsafe {
    CGEvent::integer_value_field(
      Some(event.as_ref()),
      CGEventField::KeyboardEventKeycode,
    )
  };

  let keycode = KeyCode::from_raw(key_code);

  // Try to convert to known key first, fall back to Unknown
  let pressed_key = Key::try_from(keycode).unwrap_or(Key::Unknown(keycode));

  let key_event = KeyEvent::new(pressed_key, is_keypress, event_flags);

  // ... rest of existing code ...
}
```

#### 4. Add macOS KeyEvent keycode accessor

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs`
**Changes**: Add keycode field and accessor to macOS KeyEvent

```rust
/// macOS-specific keyboard event.
#[derive(Clone, Debug)]
pub struct KeyEvent {
  /// Whether the event is for a key press or release.
  pub is_keypress: bool,

  /// The key that was pressed or released.
  pub key: Key,

  /// Modifier key flags at the time of the event.
  event_flags: CGEventFlags,

  /// Key code that generated this event.
  keycode: KeyCode,
}

impl KeyEvent {
  pub(crate) fn new(
    key: Key,
    is_keypress: bool,
    event_flags: CGEventFlags,
    keycode: KeyCode,
  ) -> Self {
    Self {
      is_keypress,
      key,
      event_flags,
      keycode,
    }
  }

  /// Gets the raw key code for this event.
  pub fn keycode(&self) -> KeyCode {
    self.keycode
  }

  // ... existing is_key_down implementation ...
}
```

### Success Criteria:

#### Automated Verification:

- [x] Code compiles: `cargo check -p wm-platform`
- [x] Unit tests pass: `cargo test -p wm-platform`
- [x] No clippy warnings: `cargo clippy -p wm-platform`

#### Manual Verification:

- [x] Unknown key codes are properly handled as `Key::Unknown`
- [x] All existing key mappings continue to work
- [x] KeyEvent provides access to raw key codes
- [x] Both platforms handle unknown keys consistently

---

### Success Criteria:

#### Automated Verification:

- [x] Code compiles on Windows: `cargo check -p wm-platform --target x86_64-pc-windows-msvc`
- [x] Unit tests pass: `cargo test -p wm-platform`
- [x] No clippy warnings: `cargo clippy -p wm-platform`

#### Manual Verification:

- [ ] Layout validation correctly identifies valid keys
- [ ] Special keys are always considered valid
- [ ] Unknown but valid keys are properly handled
- [ ] Invalid keys are logged but still processed

---

### Success Criteria:

#### Automated Verification:

- [x] Code compiles: `cargo check -p wm-platform`
- [x] All tests pass: `cargo test -p wm-platform`
- [x] No clippy warnings: `cargo clippy -p wm-platform`
- [x] Deprecation warnings appear for old function usage

#### Manual Verification:

- [ ] All conversion calls use the new trait-based API
- [ ] Unknown keys are handled consistently
- [ ] No regressions in key handling functionality
- [ ] Deprecation warnings guide users to new API

---

## Testing Strategy

### Unit Tests:

#### KeyCode Tests:

- Test `KeyCode` creation and raw value access
- Test platform-specific type conversions
- Test `Display` formatting
- Test trait implementations (PartialEq, Hash, etc.)

#### Conversion Tests:

- Test `From`/`TryFrom` implementations for all supported keys
- Test round-trip conversion consistency
- Test error cases for unsupported keys
- Test `Key::Unknown` handling

#### Windows-Specific Tests:

- Test keyboard layout validation with various layouts
- Test special key identification
- Test VK code conversion with unknown keys

### Manual Testing Steps:

1. **Basic Functionality**:

   - Verify all existing keybindings continue to work
   - Test modifier key combinations
   - Test function keys and special keys

2. **Unknown Key Handling**:

   - Test with international keyboard layouts
   - Test with custom key mappings
   - Verify unknown keys don't crash the system

3. **API Usage**:

   - Test new trait-based conversion API
   - Verify error messages are helpful
   - Test deprecated function warnings

4. **Platform-Specific**:
   - Test Windows layout validation with different layouts
   - Test macOS with different keyboard types
   - Verify platform-specific edge cases

## Performance Considerations

- `KeyCode` struct is a simple wrapper - no performance overhead
- Trait-based conversion should be zero-cost abstractions
- Layout validation on Windows adds minimal overhead (only for unknown keys)
- Unknown key handling adds one enum variant - minimal memory impact

## Migration Notes

**Breaking Changes for Internal APIs:**

- `KeyEvent` constructors now take `KeyCode` instead of raw integers
- Old conversion functions are deprecated but still functional

**Compatibility:**

- External `KeybindingListener` API remains unchanged
- All existing `Key` enum variants work as before
- Only new functionality for unknown keys

## References

- Reference patterns: `packages/wm-platform/src/native_window.rs:12-15`, `packages/wm-platform/src/display.rs:14-17`
- Current Windows key handling: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs:50`
- Current macOS key handling: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs:169-174`
- Key enum definition: `packages/wm-platform/src/models/key.rs:5-140`
- Conversion functions: Windows (`keyboard_hook.rs:102-338`), macOS (`key.rs:4-187`)
