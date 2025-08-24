# wm-platform Development Guide

This guide provides comprehensive technical details and best practices for working with the `wm-platform` crate.

## Crate Overview

The `wm-platform` crate is the foundation of GlazeWM's cross-platform functionality. It abstracts platform-specific window management, display handling, and event processing into a unified Rust API.

### Key Components

- **Platform Hooks**: Install event listeners and integrate with OS event loops
- **Event System**: Expose window events, keyboard shortcuts, and mouse interactions
- **Window Management**: Control native windows (title, visibility, resizing, process info)
- **Display Management**: Query display properties, device information, and monitor configurations

### Architecture Pattern

```
src/
├── lib.rs                  # Public API exports
├── display.rs             # Display and DisplayDevice types
├── native_window.rs       # NativeWindow type and operations
├── platform_event.rs     # Event types (WindowEvent, KeybindingEvent, etc.)
├── platform_hook.rs      # Main PlatformHook coordinator
├── platform_hook_installer.rs  # Event loop integration
├── error.rs              # Error types using thiserror
└── platform_impl/       # Platform-specific implementations
    ├── mod.rs            # Conditional compilation routing
    ├── windows/          # Windows-specific implementations
    └── macos/            # macOS-specific implementations
```

## Core API Types

### Display System

- `Display`: Represents a logical display/monitor with bounds, scale factor, DPI
- `DisplayDevice`: Physical device info (rotation, connection state, refresh rate)
- `DisplayId`: Platform-specific identifier (Windows: `HMONITOR`, macOS: `CGDirectDisplayID`)

### Window Management

- `NativeWindow`: Handle to a native OS window with title, class name, process info
- `WindowId`: Platform-specific identifier (Windows: `HWND`, macOS: `CGWindowID`)
- `ZOrder`: Window layering control (Normal, Top, TopMost, AfterWindow)

### Event System

- `PlatformEvent`: Top-level event enum (Window, Keybinding, MouseMove, DisplaySettingsChanged)
- `WindowEvent`: Window-specific events (Focus, Hide, LocationChange, Minimize, etc.)
- `KeybindingEvent`: Keyboard shortcut with key, command, and mode
- `MouseMoveEvent`: Mouse position and button state

### Platform Integration

- `PlatformHook`: Main coordinator for system integration
- `PlatformHookInstaller`: Handles event loop setup and thread management

## Implementation Patterns

### Cross-Platform Type Design

All public types follow this pattern:

```rust
pub struct PublicType {
  pub(crate) inner: platform_impl::PublicType,
}

impl PublicType {
  pub fn operation(&self) -> Result<ReturnType> {
    self.inner.operation()
  }
}
```

### Platform-Specific Identifiers

Types use conditional compilation for platform-specific backing:

```rust
pub struct TypeId(
  #[cfg(target_os = "windows")] pub(crate) windows_type,
  #[cfg(target_os = "macos")] pub(crate) macos_type,
);
```

## Platform API Integration

### Windows Integration (`windows` crate)

**Key Principles:**

- Use `windows` crate (NOT `winapi`) for Win32 API access
- Handle `HRESULT` return codes with `windows::core::Error`
- Use `PWSTR` and `PCWSTR` for wide string handling
- Manage COM object lifetimes with appropriate Release patterns

**Common Patterns:**

```rust
// String conversion
let wide_string: Vec<u16> = OsStr::new(s).encode_wide().chain(Some(0)).collect();
let pwstr = PWSTR(wide_string.as_mut_ptr());

// Error handling
let result = unsafe { SomeWin32Function(param) };
result.ok()?; // Converts HRESULT to Result

// Handle management
struct WindowsType {
  handle: HWND, // Or HMONITOR, etc.
}
```

### macOS Integration (`objc2` ecosystem)

**Key Principles:**

- Use `objc2`, `objc2-foundation`, `objc2-core-foundation`, `objc2-app-kit`, `objc2-application-services`, `objc2-core-graphics` for Cocoa APIs.

**Common Patterns:**

```rust
// String handling
let ns_string = NSString::from_str(rust_str);
let rust_string = ns_string.to_string();

// Memory management
let retained_object: Retained<NSObject> = unsafe { msg_send_id![class, method] };

// Accessibility
let ax_element = AXUIElementCreateApplication(pid);
let result = unsafe { AXUIElementGetAttributeValue(element, attribute, &mut value) };
```

## Development Guidelines

### 1. Cross-Platform API Design

- Design public APIs to be platform-agnostic
- Keep platform differences in `platform_impl/` modules

### 2. Error Handling Strategy

- Use `thiserror::Error` for custom error types
- Use `Result<T>` for all fallible operations
- Avoid `.unwrap()` - use `?` operator and proper error propagation
- Convert platform errors to crate-specific error types
- Include context in error messages

### 3. Memory Safety

- Validate all raw pointers before dereferencing
- Use RAII patterns for resource cleanup
- Add `// SAFETY:` comments for unsafe code

### 4. Thread Safety

- Use appropriate synchronization primitives
- Handle main thread requirements (especially on macOS)
- Be explicit about async boundaries

## Common Implementation Patterns

### Adding a New API

1. **Define public types in `src/`**: Add the cross-platform API surface
2. **Add platform trait in `platform_impl/`**: Define the interface each platform must implement
3. **Implement for Windows in `platform_impl/windows/`**: Use `windows` crate APIs
4. **Implement for macOS in `platform_impl/macos/`**: Use `objc2` ecosystem APIs
5. **Export from `lib.rs`**: Make the API publicly available

### Event Handling Pattern

```rust
// 1. Create platform hook
let (mut hook, installer) = PlatformHook::new();

// 2. Set up listeners before installing
let window_listener = hook.create_window_listener().await?;

// 3. Install on event loop
installer.run_dedicated_loop()?; // Blocks until shutdown
```

### Display Querying Pattern

```rust
// Get all displays
let displays = hook.displays().await?;

// Get primary display
let primary = PlatformHook::primary_display()?;

// Get display at point
let display = PlatformHook::display_from_point(point)?;
```

## Reference Material

### Platform-Specific Documentation

Use Context7 MCP server for up-to-date API docs:

- **Windows (`windows` crate)**: `/microsoft/windows-rs` and `/websites/docs_rs-windows-latest-windows`
- **macOS (`objc2` ecosystem)**:
  - Foundation: `/websites/docs_rs-objc2-foundation-latest-objc2_foundation`
  - AppKit: `/websites/docs_rs-objc2-app-kit-latest-objc2_app_kit`
  - Core Graphics: `/websites/docs_rs-objc2-core-graphics-latest-objc2_core_graphics`

### Reference Implementations

See `.claude/references/` for implementation patterns from similar projects:

- **Windows**: komorebi.md
- **macOS**: glide-wm.md, paneru.md

Note that reference projects may use other API crates like `winapi` on Windows or `core-foundation` and its related subcrates on macOS. These may have different API signatures.

You can use the GitHub MCP server to look up specific files from these references, e.g. to read `src/manager.rs` from paneru.

You can also use Grep's MCP server to search through all public GitHub repos for code snipepts. For example, "CGWindowListCopyWindowInfo objc2" for snippets that are likely using `CGWindowListCopyWindowInfo` together with `objc2`.
