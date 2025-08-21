## Glide WM macOS API usage map

Concise, per-file reference of the macOS frameworks/APIs used by `glide-wm`. This can be used as a guide for calling AppKit, CoreGraphics, CoreFoundation, Accessibility, and private SkyLight/CGS APIs from Rust.

### Crate-to-framework mapping

- core-foundation: CoreFoundation (CFRunLoop, CFArray, CFString, CFBoolean)
- core-graphics / core-graphics-types: CoreGraphics (CGDisplay, CGWindow APIs, CGError)
- objc2, objc2-app-kit, objc2-foundation, objc2-core-foundation: AppKit/Foundation/CF type bridges (e.g., `NSScreen`, `NSWindow`, `NSEvent`)
- accessibility / accessibility-sys: AX API (AXUIElement, AXObserver, roles/attributes)

## Files and APIs

### src/sys/screen.rs

- AppKit: `NSScreen::screens`, `frame()`, `visibleFrame()`
- CoreGraphics: `CGGetActiveDisplayList`, `CGDisplayBounds`
- CoreFoundation: `CFString`
- Private CGS (CoreGraphics Services, linked via CoreGraphics):
  - `CGSMainConnectionID`, `CGSGetActiveSpace`, `CGSCopySpaces`,
    `CGSCopyManagedDisplays`, `CGSCopyManagedDisplaySpaces`,
    `CGSManagedDisplayGetCurrentSpace`, `CGSCopyBestManagedDisplayForRect`
- Purpose: Compute screen layout and visible frames; convert between Quartz (top-left) and Cocoa (bottom-left) coords; query current Space IDs and active space number.

### src/sys/window_server.rs

- CoreGraphics Window Server (ApplicationServices/CoreGraphics):
  - `CGWindowListCopyWindowInfo`, `CGWindowListCreateDescriptionFromArray`,
    keys: `kCGWindowNumber`, `kCGWindowOwnerPID`, `kCGWindowLayer`, `kCGWindowBounds`
- AppKit: `NSWindow::windowNumberAtPoint_belowWindowWithWindowNumber`
- Accessibility (private): `_AXUIElementGetWindow(AXUIElementRef, *mut CGWindowID)`
- Private SkyLight (linked as SkyLight):
  - `_SLPSSetFrontProcessWithOptions`, `SLPSPostEventRecordTo` (focus/activate window)
- Private CGS (linked via ApplicationServices):
  - `CGSMainConnectionID`, `CGSSetConnectionProperty` (e.g., `"SetsCursorInBackground"`)
- CoreFoundation: `CFArray`, `CFDictionary`, `CFString`, `CFBoolean`
- Purpose: Map AX elements to Window Server IDs; enumerate visible windows; fetch window metadata; programmatically focus/raise windows; allow cursor hide from background.

### src/sys/event.rs

- AppKit: `NSEvent::pressedMouseButtons`, `NSEvent::mouseLocation`
- CoreGraphics: `CGWarpMouseCursorPosition`, `CGDisplayHideCursor`, `CGDisplayShowCursor`, `kCGNullDirectDisplayID`
- Purpose: Global hotkey integration (via external crate), mouse state/position, cursor warping and hide/show.

### src/sys/observer.rs

- Accessibility: `AXObserverCreate`, `AXObserverAddNotification`, `AXObserverRemoveNotification`, `AXObserverGetRunLoopSource`
- CoreFoundation RunLoop: `CFRunLoopAddSource`, `CFRunLoopGetCurrent`, `kCFRunLoopCommonModes`
- Purpose: Subscribe to AX notifications for a target PID and pump them via the current thread’s CFRunLoop.

### src/sys/run_loop.rs

- CoreFoundation RunLoop: `CFRunLoopSourceCreate`, `CFRunLoopSourceSignal`, `CFRunLoopWakeUp`, `CFRunLoop::get_current`, `kCFRunLoopCommonModes`
- Purpose: Create a manual CFRunLoopSource used as a wake mechanism for the custom async executor.

### src/sys/executor.rs

- CoreFoundation RunLoop: `CFRunLoop::run_current`, `CFRunLoop::get_current().stop()`
- Purpose: Minimal single-task executor that integrates Rust futures with CFRunLoop using a custom waker.

### src/sys/timer.rs

- CoreFoundation: `CFRunLoopTimer::new`, `CFRunLoop::current().add_timer`, `CFAbsoluteTimeGetCurrent`, `CFTimeInterval`, `kCFRunLoopCommonModes`
- Purpose: One-shot and repeating timers integrated with CFRunLoop, exposed as `Future`/`Stream`.

### src/sys/app.rs

- AppKit: `NSWorkspace::sharedWorkspace().runningApplications()`, `NSRunningApplication` (bundle id, pid, localized name)
- ApplicationServices (deprecated Process Manager): `GetProcessForPID`, `GetProcessInformation` (via `ProcessSerialNumber`/`ProcessInfoRec`)
- CoreFoundation: `CFString` for AX attribute names
- Purpose: Enumerate running apps, obtain per-process info (including XPC detection), convert to internal structs.

### src/actor/app.rs

- Accessibility (AXUIElement): window enumeration (`.windows()`), attributes (`role`, `subrole`, `frame`, `title`, `frontmost`, `main_window`), actions (`raise`, `set_position`, `set_size`)
- AppKit: `NSRunningApplication::with_process_id`, bundle id/name; uses AX notifications for window/app lifecycle
- Private SkyLight and Window Server: delegates to `sys/window_server.rs` to focus/raise and to read CGWindow metadata
- CoreFoundation RunLoop: used to stop the loop on termination
- Purpose: Per-application actor managing AX notifications, window tracking, raises, and event emission.

### src/actor/mouse.rs

- CoreGraphics: `CGEventTap` (session-level passive tap), `CGEventType`, event location; Mach port run loop source integration
- CoreFoundation RunLoop: add event tap source to `kCFRunLoopCommonModes`
- AppKit: uses `MainThreadMarker` for main-thread-only calls (e.g., `NSWindow::windowNumberAtPoint` via `window_server`)
- Window Server: calls into `sys/window_server` to query window at point and allow cursor hide
- Purpose: Track mouse events globally, implement focus-follows-mouse, and manage cursor hide/show based on activity.

### src/actor/notification_center.rs

- AppKit: `NSWorkspace` notifications (launch/terminate/activate/deactivate, active Space change), `NSApplicationDidChangeScreenParametersNotification`
- Foundation: `NSNotificationCenter`, selector-based observer registration
- CoreFoundation RunLoop: indirectly via `ScreenCache` updates on main thread
- Purpose: Bridge AppKit notification streams into internal `WmEvent`s; refresh screen/space state.

### src/sys/geometry.rs

- Bridging only: conversions between CoreGraphics (`core_graphics_types`) and objc2 CoreFoundation geometry types
- Purpose: Type conversions and geometry helpers (no direct system calls).

## Private API usage notes

- SkyLight `_SLPSSetFrontProcessWithOptions`, `SLPSPostEventRecordTo`: used to make a specific window key and simulate event records to ensure focus. Behavior may change across macOS versions.
- CGS functions (CoreGraphics Services): used for Space IDs, display-space mapping, and connection properties. These are undocumented and may break.
- `_AXUIElementGetWindow`: private AX to resolve AXUIElement → CGWindowID. Fallbacks should be considered.

## Coordinate systems

- Quartz/CoreGraphics: origin at top-left; AppKit/Cocoa: origin at bottom-left. See `src/sys/screen.rs::CoordinateConverter` for conversions.
