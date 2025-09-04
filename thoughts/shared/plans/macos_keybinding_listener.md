# macOS Keybinding Listener Implementation Plan

## Overview

Implement a macOS-specific keybinding listener in the wm-platform crate to enable global hotkey support for GlazeWM on macOS. The implementation will follow the existing Windows pattern but use macOS-specific APIs (CGEventTap and Carbon TIS) for global keyboard event monitoring.

## Current State Analysis

### What exists now:

- Complete Windows implementation at `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs:77-96`
- Stub macOS implementation returning `None` at `packages/wm-platform/src/keybinding_listener.rs:22-25`
- Event system architecture with `PlatformEvent::Keybinding(KeybindingEvent)`
- macOS platform abstraction using objc2 ecosystem
- Integration points in main wm application expecting `KeybindingListener::new()` at `packages/wm/src/main.rs:111`

### What's missing:

- macOS-specific keyboard event monitoring using CGEventTap
- Carbon TIS APIs for keyboard layout translation
- macOS key code to string mapping
- Integration with existing macOS event loop and dispatcher patterns

### Key constraints discovered:

- Must use CGEventTap (modern, sandbox-compatible approach)
- Requires accessibility permissions on macOS
- Must integrate with existing `MainThreadRef<T>` and `Dispatcher` patterns
- Must follow objc2 ecosystem conventions used throughout macOS platform code

## Desired End State

A fully functional macOS keybinding listener that:

- Registers global hotkeys using CGEventTap
- Translates keyboard events using Carbon TIS APIs
- Emits `KeybindingEvent` through existing event system
- Supports dynamic keybinding updates
- Integrates seamlessly with existing macOS event loop
- Follows same interface as Windows implementation

### Verification:

- Keybindings from config work globally across all applications
- Events integrate properly with main wm event processing loop
- Performance is acceptable with no noticeable input lag
- Accessibility permissions are properly requested and handled

## What We're NOT Doing

- Not changing the existing keybinding configuration format or data structures
- Not modifying the core event system architecture
- Not implementing NSEvent-based monitoring (using CGEventTap instead)
- Not supporting legacy Carbon RegisterEventHotKey APIs

## Implementation Approach

Follow the Windows implementation pattern but adapt to macOS APIs:

- Use same `KeyboardHook` interface and lifecycle methods
- Maintain same data structures (`ActiveKeybinding`, keybinding grouping)
- Use CGEventTap instead of Windows low-level hooks
- Integrate with existing macOS threading patterns (`MainThreadRef`, `Dispatcher`)

## Phase 1: Core macOS Keyboard Hook Infrastructure

### Overview

Create the basic macOS keyboard hook infrastructure using CGEventTap APIs, following existing macOS platform patterns.

### Changes Required:

#### 1. Add macOS Keyboard Hook Module

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs` (new)
**Changes**: Create complete macOS keyboard hook implementation

```rust
use crate::{
  platform_event::{KeybindingEvent, PlatformEvent},
  Error,
};
use objc2_core_foundation::{
  CFMachPortRef, CFRunLoopRef, CFRunLoopSourceRef,
  CFMachPortCreate, CFMachPortCreateRunLoopSource,
};
use objc2_core_graphics::{
  CGEventTapCreate, CGEventTapEnable, CGEventTapLocation,
  CGEventTapPlacement, CGEventTapOptions, CGEventType, CGEvent,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use wm_common::KeybindingConfig;

#[derive(Debug)]
pub struct KeyboardHook {
  /// Sender to emit platform events.
  event_tx: mpsc::UnboundedSender<PlatformEvent>,

  /// CGEventTap handle for keyboard monitoring.
  event_tap: Arc<Mutex<Option<CFMachPortRef>>>,

  /// RunLoop source for event tap.
  run_loop_source: Arc<Mutex<Option<CFRunLoopSourceRef>>>,

  /// Active keybindings grouped by trigger key.
  keybindings_by_trigger_key: Arc<Mutex<HashMap<u16, Vec<ActiveKeybinding>>>>,
}

#[derive(Debug, Clone)]
pub struct ActiveKeybinding {
  pub key_codes: Vec<u16>,
  pub config: KeybindingConfig,
}
```

#### 2. Add CGEventTap Callback Handler

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs`
**Changes**: Implement CGEventTap callback for keyboard event processing

```rust
unsafe extern "C" fn keyboard_event_callback(
  proxy: CGEventTapProxy,
  event_type: CGEventType,
  event: CGEventRef,
  user_info: *mut c_void,
) -> CGEventRef {
  if event_type != kCGEventKeyDown {
    return event;
  }

  let hook = &*(user_info as *const KeyboardHook);
  let key_code = CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode) as u16;

  let should_block = hook.handle_key_event(key_code);

  if should_block {
    return std::ptr::null_mut(); // Block the event
  }

  event
}
```

#### 3. Add Key Code Translation

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs`
**Changes**: Add Carbon TIS APIs for keyboard layout translation

```rust
use objc2_core_foundation::{
  TISCopyCurrentASCIICapableKeyboardLayoutInputSource,
  TISGetInputSourceProperty, kTISPropertyUnicodeKeyLayoutData,
};
use std::os::raw::c_void;

impl KeyboardHook {
  fn key_code_to_string(key_code: u16) -> Option<String> {
    unsafe {
      let keyboard_layout = TISCopyCurrentASCIICapableKeyboardLayoutInputSource();
      if keyboard_layout.is_null() {
        return None;
      }

      let layout_data = TISGetInputSourceProperty(
        keyboard_layout,
        kTISPropertyUnicodeKeyLayoutData
      );

      // Use UCKeyTranslate to convert key code to string
      // Implementation details...
    }
  }

  fn string_to_key_code(key: &str) -> Option<u16> {
    match key.to_lowercase().as_str() {
      "a" => Some(0x00),
      "s" => Some(0x01),
      "d" => Some(0x02),
      // ... complete mapping
      "cmd" | "command" => Some(0x37),
      "option" | "alt" => Some(0x3A),
      "control" | "ctrl" => Some(0x3B),
      "shift" => Some(0x38),
      _ => None,
    }
  }
}
```

#### 4. Update macOS Module Exports

**File**: `packages/wm-platform/src/platform_impl/macos/mod.rs`
**Changes**: Export the new keyboard hook module

```rust
pub use keyboard_hook::KeyboardHook;

#[cfg(target_os = "macos")]
mod keyboard_hook;
```

### Success Criteria:

#### Automated Verification:

- [x] Code compiles without errors: `cargo check --package wm-platform`
- [ ] All tests pass: `cargo test --package wm-platform`
- [ ] No clippy warnings: `cargo clippy --package wm-platform`
- [x] CGEventTap can be created without errors

#### Manual Verification:

- [ ] Accessibility permission dialog appears when needed
- [ ] Basic key events are captured (verified with debug logging)
- [ ] No system performance degradation
- [ ] Event tap can be enabled and disabled properly

---

## Phase 2: Keybinding Registration and Event Processing

### Overview

Implement keybinding registration, matching, and event emission following the Windows implementation pattern.

### Changes Required:

#### 1. Add Keybinding Registration Logic

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs`
**Changes**: Implement keybinding parsing and organization

```rust
impl KeyboardHook {
  pub fn new(
    keybindings: &Vec<KeybindingConfig>,
    event_tx: mpsc::UnboundedSender<PlatformEvent>,
  ) -> crate::Result<Arc<Self>> {
    let keyboard_hook = Arc::new(Self {
      event_tx,
      event_tap: Arc::new(Mutex::new(None)),
      run_loop_source: Arc::new(Mutex::new(None)),
      keybindings_by_trigger_key: Arc::new(Mutex::new(
        Self::keybindings_by_trigger_key(keybindings)
      )),
    });

    Ok(keyboard_hook)
  }

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
}
```

#### 2. Add Event Matching Logic

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs`
**Changes**: Implement key event matching and filtering

```rust
impl KeyboardHook {
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

        if let Some(longest_binding) = matched_bindings.max_by_key(|b| b.key_codes.len()) {
          let _ = self.event_tx.send(PlatformEvent::Keybinding(
            KeybindingEvent {
              key: longest_binding.config.bindings[0].clone(),
              command: longest_binding.config.commands[0].command.clone(),
              mode: "".to_string(), // TODO: Handle binding modes
            }
          ));
          return true;
        }

        false
      }
    }
  }

  fn is_modifier_pressed(key_code: u16) -> bool {
    unsafe {
      match key_code {
        0x37 => CGEventSourceFlagsState(kCGEventSourceStatePrivate) & kCGEventFlagMaskCommand != 0,
        0x3A => CGEventSourceFlagsState(kCGEventSourceStatePrivate) & kCGEventFlagMaskAlternate != 0,
        0x3B => CGEventSourceFlagsState(kCGEventSourceStatePrivate) & kCGEventFlagMaskControl != 0,
        0x38 => CGEventSourceFlagsState(kCGEventSourceStatePrivate) & kCGEventFlagMaskShift != 0,
        _ => false,
      }
    }
  }
}
```

#### 3. Add Lifecycle Management

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs`
**Changes**: Implement start/stop/update methods

```rust
impl KeyboardHook {
  pub fn start(&self) -> crate::Result<()> {
    unsafe {
      let event_mask = 1 << kCGEventKeyDown;
      let event_tap = CGEventTapCreate(
        kCGSessionEventTap,
        kCGHeadInsertEventTap,
        kCGEventTapOptionDefault,
        event_mask,
        keyboard_event_callback,
        self as *const Self as *mut c_void,
      );

      if event_tap.is_null() {
        return Err(Error::Platform("Failed to create CGEventTap".to_string()));
      }

      let run_loop_source = CFMachPortCreateRunLoopSource(
        std::ptr::null(),
        event_tap,
        0,
      );

      let run_loop = CFRunLoopGetMain();
      CFRunLoopAddSource(run_loop, run_loop_source, kCFRunLoopCommonModes);

      CGEventTapEnable(event_tap, true);

      *self.event_tap.lock().unwrap() = Some(event_tap);
      *self.run_loop_source.lock().unwrap() = Some(run_loop_source);
    }

    Ok(())
  }

  pub fn stop(&self) -> crate::Result<()> {
    // Implementation for cleanup
    Ok(())
  }

  pub fn update(&self, keybindings: &Vec<KeybindingConfig>) -> crate::Result<()> {
    *self.keybindings_by_trigger_key.lock().unwrap() =
      Self::keybindings_by_trigger_key(keybindings);
    Ok(())
  }
}
```

### Success Criteria:

#### Automated Verification:

- [x] Code compiles without errors: `cargo check --package wm-platform`
- [ ] All tests pass: `cargo test --package wm-platform`
- [x] KeybindingConfig parsing works correctly
- [x] Event emission works through tokio channels

#### Manual Verification:

- [ ] Keybindings from sample config are registered correctly
- [ ] Simple keybindings (e.g., "cmd+h") trigger events
- [ ] Complex keybindings (e.g., "cmd+shift+h") trigger events
- [ ] Longest matching keybinding wins when multiple match
- [ ] No false positive triggers from partial key combinations

---

## Phase 3: Integration with macOS Event System

### Overview

Integrate the keyboard hook with the existing macOS event loop and platform abstractions.

### Changes Required:

#### 1. Update KeybindingListener Implementation

**File**: `packages/wm-platform/src/keybinding_listener.rs`
**Changes**: Replace stub implementation with platform-specific code

```rust
#[cfg(target_os = "macos")]
impl KeybindingListener {
  pub fn new(
    keybindings: &Vec<KeybindingConfig>,
    event_tx: mpsc::UnboundedSender<PlatformEvent>,
  ) -> crate::Result<Option<Self>> {
    use crate::platform_impl::macos::KeyboardHook;

    let keyboard_hook = KeyboardHook::new(keybindings, event_tx)?;
    keyboard_hook.start()?;

    Ok(Some(Self {
      keyboard_hook: Some(keyboard_hook),
    }))
  }

  pub fn update(&self, keybindings: &Vec<KeybindingConfig>) -> crate::Result<()> {
    if let Some(ref hook) = self.keyboard_hook {
      hook.update(keybindings)?;
    }
    Ok(())
  }
}

#[cfg(target_os = "macos")]
pub struct KeybindingListener {
  keyboard_hook: Option<Arc<crate::platform_impl::macos::KeyboardHook>>,
}
```

#### 2. Add Cargo Dependencies

**File**: `packages/wm-platform/Cargo.toml`
**Changes**: Add required macOS-specific dependencies

```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2-core-foundation = { version = "0.2.2", features = [
  "CFMachPort",
  "CFRunLoop",
  "CFString",
  "TISInputSource",
] }
objc2-core-graphics = { version = "0.2.2", features = [
  "CGEvent",
  "CGEventSource",
  "CGEventTap",
] }
```

#### 3. Add FFI Declarations

**File**: `packages/wm-platform/src/platform_impl/macos/ffi.rs`
**Changes**: Add missing CGEventTap and TIS function declarations

```rust
use objc2_core_foundation::{CFMachPortRef, CFRunLoopSourceRef};
use objc2_core_graphics::{CGEventTapProxy, CGEventType, CGEventRef};

#[link(name = "Carbon", kind = "framework")]
extern "C" {
  pub fn TISCopyCurrentASCIICapableKeyboardLayoutInputSource() -> TISInputSourceRef;
  pub fn TISGetInputSourceProperty(
    source: TISInputSourceRef,
    property: CFStringRef,
  ) -> *const c_void;
  pub static kTISPropertyUnicodeKeyLayoutData: CFStringRef;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
  pub fn CGEventTapCreate(
    tap: CGEventTapLocation,
    place: CGEventTapPlacement,
    options: CGEventTapOptions,
    events_of_interest: CGEventMask,
    callback: CGEventTapCallBack,
    user_info: *mut c_void,
  ) -> CFMachPortRef;
}
```

### Success Criteria:

#### Automated Verification:

- [x] All dependencies resolve: `cargo check --package wm-platform`
- [x] FFI bindings compile correctly
- [x] KeybindingListener::new() returns Some() on macOS
- [ ] Tests pass: `cargo test --package wm-platform`

#### Manual Verification:

- [ ] KeybindingListener integrates with main wm event loop
- [ ] Events flow properly from CGEventTap to wm processing
- [ ] No memory leaks or crashes during extended usage
- [ ] Clean shutdown when application exits

---

## Phase 4: Testing and Polish

### Overview

Add comprehensive testing, error handling, and performance optimization.

### Changes Required:

#### 1. Add Unit Tests

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs`
**Changes**: Add comprehensive test coverage

```rust
#[cfg(test)]
mod tests {
  use super::*;
  use wm_common::{InvokeCommand, KeybindingConfig};

  #[test]
  fn test_keybinding_parsing() {
    let keybindings = vec![KeybindingConfig {
      bindings: vec!["cmd+h".to_string()],
      commands: vec![InvokeCommand {
        command: "focus --direction left".to_string(),
        args: None,
      }],
    }];

    let grouped = KeyboardHook::keybindings_by_trigger_key(&keybindings);
    let h_key = KeyboardHook::string_to_key_code("h").unwrap();

    assert!(grouped.contains_key(&h_key));
    assert_eq!(grouped[&h_key].len(), 1);
  }

  #[test]
  fn test_key_code_mapping() {
    assert_eq!(KeyboardHook::string_to_key_code("a"), Some(0x00));
    assert_eq!(KeyboardHook::string_to_key_code("cmd"), Some(0x37));
    assert_eq!(KeyboardHook::string_to_key_code("invalid"), None);
  }
}
```

#### 2. Add Error Handling

**File**: `packages/wm-platform/src/error.rs`
**Changes**: Add macOS-specific error types

```rust
#[cfg(target_os = "macos")]
#[error("CGEventTap operation failed: {0}")]
CGEventTap(String),

#[cfg(target_os = "macos")]
#[error("Accessibility permission required")]
AccessibilityPermission,

#[cfg(target_os = "macos")]
#[error("Keyboard layout translation failed: {0}")]
KeyboardLayout(String),
```

#### 3. Add Permission Checking

**File**: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs`
**Changes**: Add accessibility permission checking

```rust
impl KeyboardHook {
  pub fn check_accessibility_permission() -> bool {
    unsafe {
      AXIsProcessTrusted()
    }
  }

  pub fn request_accessibility_permission() -> crate::Result<()> {
    unsafe {
      let options = CFDictionaryCreateMutable(
        std::ptr::null(),
        0,
        &kCFTypeDictionaryKeyCallBacks,
        &kCFTypeDictionaryValueCallBacks,
      );

      CFDictionaryAddValue(
        options,
        kAXTrustedCheckOptionPrompt as *const c_void,
        kCFBooleanTrue as *const c_void,
      );

      AXIsProcessTrustedWithOptions(options);
      CFRelease(options);
    }

    Ok(())
  }
}
```

### Success Criteria:

#### Automated Verification:

- [ ] All unit tests pass: `cargo test --package wm-platform keyboard_hook`
- [ ] Code coverage above 80% for keyboard hook module
- [ ] No memory leaks detected: `cargo test --package wm-platform -- --test-threads=1`
- [ ] Performance benchmarks within acceptable range

#### Manual Verification:

- [ ] Accessibility permission dialog works correctly
- [ ] Error messages are user-friendly and actionable
- [ ] Keybinding conflicts are handled gracefully
- [ ] No system instability under stress testing
- [ ] Works correctly across different keyboard layouts

---

## Testing Strategy

### Unit Tests:

- Keybinding parsing and grouping logic
- Key code to string mapping bidirectional conversion
- Event matching algorithm correctness
- Error handling for invalid configurations

### Manual Testing Steps:

1. Install GlazeWM with macOS keybinding listener
2. Configure sample keybindings in YAML config
3. Test each keybinding category (focus, move, resize, workspace)
4. Verify global hotkeys work across all applications
5. Test accessibility permission flow for new installations
6. Verify clean shutdown and resource cleanup

## Performance Considerations

- **Event Filtering**: Only monitor key down events to minimize overhead
- **Efficient Matching**: Use HashMap grouping by trigger key for O(1) lookup
- **Memory Management**: Proper CFRetain/CFRelease for Core Foundation objects
- **Thread Safety**: All shared state protected by Arc<Mutex<>>
- **Minimal Allocations**: Reuse data structures where possible

## Migration Notes

No migration required - this is a new feature implementation. Existing Windows users are unaffected.

## References

- Original Windows implementation: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs`
- macOS platform patterns: `packages/wm-platform/src/platform_impl/macos/`
- Reference implementations: `thoughts/references/paneru.md`, `thoughts/references/glide-wm.md`
- Platform guide: `thoughts/wm-platform-guide.md`
