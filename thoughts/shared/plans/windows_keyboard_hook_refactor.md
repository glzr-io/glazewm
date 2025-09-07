# Windows Keyboard Hook API Refactor Implementation Plan

## Overview

Refactor the Windows keyboard hook implementation to follow a similar callback-based API to the macOS implementation, using thread-local storage instead of global state and providing a unified `KeyEvent` interface across platforms.

## Current State Analysis

**Windows Implementation Issues:**
- Uses global state (`OnceLock<Arc<KeyboardHook>>`) limiting to one hook per process
- Tightly coupled with keybinding logic - takes `Vec<KeybindingConfig>` directly
- No `KeyEvent` abstraction like macOS
- Complex constructor mixing low-level hook management with business logic

**macOS Implementation (Good Pattern):**
- Clean callback API: `KeyboardHook::new(dispatcher, callback)` where callback is `Fn(KeyEvent) -> bool`
- Has `KeyEvent` struct with `is_keypress`, `key`, and modifier state query methods
- Separates concerns: hook handles low-level events, callback handles business logic

## Desired End State

After this refactor:
1. Windows `KeyboardHook` will have similar API to macOS version
2. Use thread-local storage pattern instead of global state  
3. Windows will have its own `KeyEvent` struct similar to macOS
4. `KeybindingListener` will work uniformly across platforms
5. Support multiple hooks per thread (following proposed API pattern)

### Key Discoveries:

- macOS uses `CGEventTap` with callback pattern at `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs:76-89`
- Windows currently uses global state limiting flexibility at `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs:42-45`
- `KeybindingListener` abstracts platform differences at `packages/wm-platform/src/keybinding_listener.rs:103-179`

## What We're NOT Doing

- Not changing the external API of `KeybindingListener`
- Not modifying keybinding matching logic or business rules
- Not altering the macOS implementation
- Not changing the cross-platform `Key` enum structure
- Not modifying the event dispatching system

## Implementation Approach

Use incremental refactoring to transform Windows implementation piece by piece while maintaining functionality. Each phase builds on the previous one with clear rollback points.

## Phase 1: Create Windows KeyEvent Structure

### Overview

Create a Windows-specific `KeyEvent` struct that provides the same interface as the macOS version.

### Changes Required:

#### 1. Windows KeyEvent Implementation

**File**: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs`
**Changes**: Add new `KeyEvent` struct and supporting types

```rust
/// Windows-specific keyboard event.
#[derive(Clone, Debug)]
pub struct KeyEvent {
    /// Whether the event is for a key press or release.
    pub is_keypress: bool,

    /// The key that was pressed or released.
    pub key: Key,

    /// Virtual key code of the pressed key.
    vk_code: u16,
}

impl KeyEvent {
    /// Creates an instance of `KeyEvent`.
    pub(crate) fn new(key: Key, is_keypress: bool, vk_code: u16) -> Self {
        Self {
            is_keypress,
            key,
            vk_code,
        }
    }

    /// Gets whether the specified key is currently pressed.
    pub fn is_key_down(&self, key: Key) -> bool {
        match key {
            Key::Cmd => {
                Self::is_key_down_raw(VK_LWIN.0) || Self::is_key_down_raw(VK_RWIN.0)
            }
            Key::Alt => {
                Self::is_key_down_raw(VK_LMENU.0) || Self::is_key_down_raw(VK_RMENU.0)
            }
            Key::Ctrl => {
                Self::is_key_down_raw(VK_LCONTROL.0) || Self::is_key_down_raw(VK_RCONTROL.0)
            }
            Key::Shift => {
                Self::is_key_down_raw(VK_LSHIFT.0) || Self::is_key_down_raw(VK_RSHIFT.0)
            }
            _ => {
                if let Some(vk_code) = key_to_vk_code(key) {
                    Self::is_key_down_raw(vk_code)
                } else {
                    false
                }
            }
        }
    }

    /// Gets whether the specified key is currently down using the raw key code.
    fn is_key_down_raw(key: u16) -> bool {
        unsafe { (GetKeyState(key.into()) & 0x80) == 0x80 }
    }
}
```

#### 2. Add VK code to Key conversion

**File**: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs`
**Changes**: Extract and improve the VK code conversion logic

```rust
/// Convert Windows virtual key code to cross-platform Key enum.
fn vk_code_to_key(vk_code: u16) -> Option<Key> {
    match VIRTUAL_KEY(vk_code) {
        VK_A => Some(Key::A),
        VK_B => Some(Key::B),
        VK_C => Some(Key::C),
        // ... (map all supported keys)
        VK_LWIN => Some(Key::LCmd),
        VK_RWIN => Some(Key::RCmd),
        VK_LCONTROL => Some(Key::LCtrl),
        VK_RCONTROL => Some(Key::RCtrl),
        // ... continue for all keys
        _ => None,
    }
}

/// Convert cross-platform Key enum to Windows virtual key code.
fn key_to_vk_code(key: Key) -> Option<u16> {
    match key {
        Key::A => Some(VK_A.0),
        Key::B => Some(VK_B.0),
        Key::C => Some(VK_C.0),
        // ... (map all supported keys)
        Key::LCmd => Some(VK_LWIN.0),
        Key::RCmd => Some(VK_RWIN.0),
        Key::LCtrl => Some(VK_LCONTROL.0),
        Key::RCtrl => Some(VK_RCONTROL.0),
        // ... continue for all keys
        _ => None,
    }
}
```

### Success Criteria:

#### Automated Verification:
- [x] Code compiles successfully: `cargo check -p wm-platform --target x86_64-pc-windows-msvc`
- [x] Unit tests pass for key conversion: `cargo test vk_code_to_key`
- [x] No clippy warnings: `cargo clippy -p wm-platform --target x86_64-pc-windows-msvc`

#### Manual Verification:
- [ ] `KeyEvent::is_key_down()` correctly reports modifier key states
- [ ] Key conversion handles all supported keys correctly
- [ ] VK code conversion is bidirectional and consistent

---

## Phase 2: Implement Thread-Local Hook Storage

### Overview

Replace the global `OnceLock` with thread-local storage following the proposed pattern.

### Changes Required:

#### 1. Replace global state with thread-local storage

**File**: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs`
**Changes**: Implement thread-local pattern

```rust
use std::cell::Cell;

thread_local! {
    /// Stores the hook callback for the current thread.
    static HOOK: Cell<Option<Box<HookFn>>> = Cell::default();
}

type HookFn = dyn FnMut(KeyEvent) -> bool;
```

#### 2. Refactor KeyboardHook struct

**File**: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs`  
**Changes**: Simplify to match macOS API

```rust
/// Wrapper for the low-level keyboard hook API.
/// Automatically unregisters the hook when dropped.
pub struct KeyboardHook {
    handle: HHOOK,
}

impl KeyboardHook {
    /// Sets the low-level keyboard hook for this thread.
    ///
    /// Panics when a hook is already registered from the same thread.
    #[must_use = "The hook will immediately be unregistered and not work."]
    pub fn new<F>(dispatcher: Dispatcher, callback: F) -> crate::Result<Self>
    where
        F: FnMut(KeyEvent) -> bool + 'static,
    {
        HOOK.with(|state| {
            assert!(
                state.take().is_none(),
                "Only one keyboard hook can be registered per thread."
            );

            state.set(Some(Box::new(callback)));

            let handle = unsafe {
                SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), ptr::null_mut(), 0)
                    .as_mut()
                    .expect("install low-level keyboard hook successfully")
            };

            Ok(KeyboardHook { handle })
        })
    }

    /// Stops the keyboard hook by unregistering it.
    pub fn stop(&mut self) -> crate::Result<()> {
        unsafe { UnhookWindowsHookEx(self.handle) }?;
        HOOK.with(|state| { state.take(); });
        Ok(())
    }
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
```

#### 3. Update hook procedure

**File**: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs`
**Changes**: Simplify hook procedure to use thread-local callback

```rust
extern "system" fn hook_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // Early return for ignored events
    if code < 0 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    let should_ignore = !(wparam.0 as u32 == WM_KEYDOWN
        || wparam.0 as u32 == WM_SYSKEYDOWN);

    if should_ignore {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    let input = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let vk_code = input.vkCode as u16;
    
    if let Some(key) = vk_code_to_key(vk_code) {
        let key_event = KeyEvent::new(key, true, vk_code);
        
        let should_intercept = HOOK.with(|state| {
            if let Some(mut callback) = state.take() {
                let result = callback(key_event);
                state.set(Some(callback));
                result
            } else {
                false
            }
        });

        if should_intercept {
            return LRESULT(1);
        }
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}
```

### Success Criteria:

#### Automated Verification:
- [x] Code compiles: `cargo check -p wm-platform --target x86_64-pc-windows-msvc`
- [x] No clippy warnings: `cargo clippy -p wm-platform --target x86_64-pc-windows-msvc`
- [x] Unit tests pass: `cargo test -p wm-platform`

#### Manual Verification:
- [ ] Multiple hooks can be created on different threads
- [ ] Hook cleanup works correctly when dropped
- [ ] Callback receives proper `KeyEvent` instances
- [ ] Thread-local storage prevents interference between hooks

---

## Phase 3: Update KeybindingListener and Remove Windows-Specific Logic

### Overview

Remove Windows-specific keybinding logic from `KeyboardHook` and update `KeybindingListener` to work with the new unified API.

### Changes Required:

#### 1. Remove keybinding logic from Windows KeyboardHook

**File**: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs`
**Changes**: Remove all keybinding-specific code

- Remove `ActiveKeybinding` struct (lines 57-61)
- Remove `keybindings_by_trigger_key` field from `KeyboardHook` (lines 73-74)
- Remove `keybindings_by_trigger_key()` method (lines 133-170)
- Remove `update()` method (lines 118-121)
- Remove `handle_key_event()` method (lines 325-401)
- Remove keybinding-specific imports and constructor parameters

#### 2. Update Windows module exports

**File**: `packages/wm-platform/src/platform_impl/windows/mod.rs`
**Changes**: Export the new `KeyEvent` type

```rust
pub use keyboard_hook::{KeyboardHook, KeyEvent};
```

#### 3. Verify KeybindingListener compatibility

**File**: `packages/wm-platform/src/keybinding_listener.rs`
**Changes**: Should work with both platforms without modification

The callback function in `create_keyboard_hook` (lines 105-179) should already work with both `KeyEvent` types since they have the same interface.

#### 4. Remove Windows-specific imports

**File**: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs`
**Changes**: Clean up unused imports

Remove unused imports related to keybinding logic:
- `KeybindingConfig` import
- `PlatformEvent` import
- Any other keybinding-specific dependencies

### Success Criteria:

#### Automated Verification:
- [x] Full compilation succeeds: `cargo check`
- [x] All tests pass: `cargo test -p wm-platform`  
- [x] Linting passes: `cargo clippy -p wm-platform`
- [x] No unused import warnings

#### Manual Verification:
- [ ] `KeybindingListener` works identically on both platforms
- [ ] Keybinding matching logic works correctly
- [ ] Hook cleanup and updates work properly
- [ ] No regressions in keybinding functionality

---

## Phase 4: Final Integration and Cleanup

### Overview

Ensure the new Windows implementation integrates properly with the existing event system and clean up any remaining inconsistencies.

### Changes Required:

#### 1. Verify platform_impl module structure

**File**: `packages/wm-platform/src/platform_impl/mod.rs`
**Changes**: Ensure `KeyEvent` is properly re-exported on both platforms

#### 2. Update any remaining references

**Changes**: Search for and update any remaining platform-specific references

```bash
# Search for potential issues
rg "KeyboardHook" packages/wm-platform/src/
rg "KeybindingConfig" packages/wm-platform/src/platform_impl/windows/
```

#### 3. Add comprehensive documentation

**File**: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs`
**Changes**: Add rustdoc comments following project guidelines

```rust
/// Windows-specific keyboard event.
///
/// Provides a unified interface for keyboard events across the Windows platform,
/// compatible with the cross-platform `KeybindingListener`.
///
/// # Platform-specific
///
/// Uses Windows `GetKeyState` API for modifier key state detection.
#[derive(Clone, Debug)]
pub struct KeyEvent {
    // ... implementation
}

/// Wrapper for the low-level keyboard hook API.
///
/// Automatically unregisters the hook when dropped to prevent resource leaks.
/// Uses thread-local storage to support multiple independent hooks per thread.
///
/// # Panics
///
/// Panics when attempting to register multiple hooks on the same thread.
///
/// # Platform-specific
///
/// Uses Windows `SetWindowsHookExW` with `WH_KEYBOARD_LL` for system-wide
/// keyboard event interception.
pub struct KeyboardHook {
    // ... implementation
}
```

### Success Criteria:

#### Automated Verification:
- [x] Full project builds: `cargo build`
- [x] All tests pass: `cargo test`
- [x] No compiler warnings: `cargo check`
- [x] Documentation builds: `cargo doc -p wm-platform`

#### Manual Verification:
- [ ] Both platforms have identical `KeyboardHook` API surface
- [ ] Event flow works end-to-end on both platforms
- [ ] No regressions in keybinding functionality
- [ ] Documentation is comprehensive and follows project standards

---

## Testing Strategy

### Unit Tests:

- Test `vk_code_to_key` conversion with various Windows virtual key codes  
- Test `KeyEvent::is_key_down` with different modifier combinations
- Test thread-local hook storage with multiple hook instances
- Test hook cleanup on drop
- Test bidirectional key conversion consistency

### Integration Tests:

- Test full keybinding flow with new Windows implementation
- Test that multiple threads can have independent hooks
- Verify identical behavior between Windows and macOS implementations
- Test hook registration and cleanup edge cases

### Manual Testing Steps:

1. Verify keybindings still work correctly in GlazeWM after refactor
2. Test that modifier key combinations are handled properly  
3. Confirm no memory leaks or resource issues with hook cleanup
4. Verify thread safety with multiple hook instances
5. Test edge cases like rapid hook creation/destruction
6. Verify accessibility permissions work correctly

## Performance Considerations

- Thread-local storage should be more performant than global locking
- Reduced indirection in hook procedure callback path  
- Eliminated redundant keybinding processing in hook vs listener
- Simplified hook procedure reduces per-event overhead

## Migration Notes

This is a refactoring change that should not affect the external API of `KeybindingListener`. The changes are contained within the `wm-platform` crate and should be transparent to consumers.

**Breaking changes**: None for public APIs
**Internal changes**: Complete refactor of Windows keyboard hook implementation

## References

- Current macOS implementation: `packages/wm-platform/src/platform_impl/macos/keyboard_hook.rs:76-89`
- Current Windows implementation: `packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs:77-96`
- Proposed API pattern: Thread-local storage with callback-based API
- Usage in KeybindingListener: `packages/wm-platform/src/keybinding_listener.rs:103-179`
- Cross-platform Key enum: `packages/wm-platform/src/models/key.rs`