# wm-platform API Redesign Specification

## Overview

This specification outlines the redesign of the `wm-platform` crate's top-level API, replacing `PlatformHook` and `PlatformHookInstaller` with a cleaner, more intuitive design centered around `EventLoop`, `EventLoopInstaller`, and `Dispatcher`.

## New API Design

### Core Types

```rust
pub struct EventLoop {
    // Platform-specific event loop implementation
}

pub struct EventLoopInstaller {
    // Installer for integrating with existing event loops
}

pub struct Dispatcher {
    // Thread-safe dispatcher for cross-platform operations
    // Implements: Clone + Send + Sync
}

// Listener types
pub struct MouseListener { /* ... */ }
pub struct WindowListener { /* ... */ }
pub struct KeybindingListener { /* ... */ }
```

### Usage Patterns

```rust
// Pattern 1: Dedicated event loop
let (event_loop, dispatcher) = EventLoop::new()?;

let task_handle = rt.spawn(async {
    let windows = dispatcher.visible_windows()?;
    let mouse_listener = MouseListener::new(dispatcher.clone())?;
    let window_listener = WindowListener::new(dispatcher)?;

    // ... application logic
});

event_loop.run(); // Blocks until shutdown

// Pattern 2: Integration with existing event loop
let (installer, dispatcher) = EventLoopInstaller::new()?;

let task_handle = rt.spawn(async {
    let windows = dispatcher.visible_windows()?;
    let mouse_listener = MouseListener::new(dispatcher)?;
    // ... application logic
});

// Platform-specific installation
#[cfg(target_os = "macos")]
installer.install()?;

#[cfg(target_os = "windows")]
installer.install_with_subclass(hwnd)?;
```

## API Surface

### EventLoop

```rust
impl EventLoop {
    /// Creates a new event loop and dispatcher.
    pub fn new() -> (Self, Dispatcher);

    /// Runs the event loop, blocking until shutdown.
    ///
    /// # Platform-specific
    ///
    /// - **macOS**: Must be called from the main thread. Runs `CFRunLoopRun()`.
    /// - **Windows**: Can be called from any thread. Runs Win32 message loop.
    pub fn run(self);
}
```

### EventLoopInstaller

```rust
impl EventLoopInstaller {
    /// Creates a new installer and dispatcher for integrating with existing event loops.
    pub fn new() -> (Self, Dispatcher);

    /// Install on the main thread (macOS only).
    ///
    /// # Platform-specific
    ///
    /// - **macOS**: Must be called from the main thread.
    #[cfg(target_os = "macos")]
    pub fn install(self) -> crate::Result<()>;

    /// Install on an existing event loop via window subclassing (Windows only).
    ///
    /// # Platform-specific
    ///
    /// - **Windows**: Integrates with existing message loop via subclassing.
    #[cfg(target_os = "windows")]
    pub fn install_with_subclass(self, hwnd: HWND) -> crate::Result<()>;
}
```

### Dispatcher

```rust
impl Dispatcher {
    // Display queries
    pub fn displays(&self) -> crate::Result<Vec<Display>>;
    pub fn all_display_devices(&self) -> crate::Result<Vec<DisplayDevice>>;
    pub fn display_from_point(&self, point: Point) -> crate::Result<Display>;
    pub fn primary_display(&self) -> crate::Result<Display>;

    // Window queries
    pub fn all_windows(&self) -> crate::Result<Vec<NativeWindow>>;
    pub fn all_applications(&self) -> crate::Result<Vec<NativeWindow>>;
    pub fn visible_windows(&self) -> crate::Result<Vec<NativeWindow>>;
}

// Thread safety traits
impl Clone for Dispatcher { /* ... */ }
unsafe impl Send for Dispatcher {}
unsafe impl Sync for Dispatcher {}
```

### Listeners

```rust
impl MouseListener {
    /// Creates a new mouse listener using the provided dispatcher.
    pub fn new(dispatcher: Dispatcher) -> crate::Result<Self>;
}

impl WindowListener {
    /// Creates a new window listener using the provided dispatcher.
    pub fn new(dispatcher: Dispatcher) -> crate::Result<Self>;
}

impl KeybindingListener {
    /// Creates a new keybinding listener using the provided dispatcher.
    pub fn new(dispatcher: Dispatcher) -> crate::Result<Self>;
}
```

## Implementation Plan

### Phase 1: New Core Implementation

1. **Create new files**:

   - `src/event_loop.rs` - `EventLoop` implementation
   - `src/event_loop_installer.rs` - `EventLoopInstaller` implementation
   - `src/dispatcher.rs` - `Dispatcher` implementation.

2. **Dispatcher methods**:
   - Implement sync methods that internally use `Dispatcher::dispatch_sync()`
   - Move logic from current `PlatformHook` async methods
   - Ensure proper error handling with `crate::Result<T>`

### Phase 2: Listener Updates

4. **Update listener constructors**:

   - Change to take `Dispatcher` by value (enables clean ownership)
   - Remove async requirements from construction
   - Update internal implementations to use owned dispatcher

5. **Platform-specific installer methods**:
   - Implement `install()` for macOS (verify main thread, integrate with existing CFRunLoop)
   - Implement `install_with_subclass()` for Windows (set up window subclassing)

### Phase 3: Integration

6. **Update public exports in `lib.rs`**:

   - Remove exports of `PlatformHook` and `PlatformHookInstaller`
   - Add exports for new types: `EventLoop`, `EventLoopInstaller`, `Dispatcher`
   - Update doc comments

7. **Update platform implementations**:

   - Ensure `platform_impl::Dispatcher` supports the new API requirements
   - Test cross-platform behavior
   - Verify thread safety guarantees are maintained

8. **Remove old implementation**:
   - Delete `src/platform_hook.rs` and `src/platform_hook_installer.rs`
   - Clean up any internal dependencies on the old types
   - Update any remaining references

## Technical Considerations

### Thread Safety

- `Dispatcher` wraps the existing `Dispatcher` which is already `Send + Sync`
- All operations are thread-safe via internal dispatch mechanisms
- macOS operations will automatically dispatch to main thread when necessary

### Error Handling

- All public methods return `crate::Result<T>` for consistency

### Memory Management

- `Dispatcher` is cheap to clone (Arc-based internally)
- Listeners take ownership of dispatcher to avoid lifetime complexity
- No breaking changes to existing memory management patterns

## Migration Impact

### Breaking Changes

- Complete replacement of `PlatformHook` and `PlatformHookInstaller` APIs
- Listener construction changes from async factory methods to sync constructors
- Query operations change from async to sync (but maintain cross-thread dispatch internally)
