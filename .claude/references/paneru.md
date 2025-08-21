## Paneru macOS API usage map

Concise, per-file reference of the macOS frameworks/APIs used by `paneru`. Intended as a guide for calling AppKit, CoreGraphics, CoreFoundation, Accessibility (AX), Carbon, and private SkyLight/CGS APIs from Rust.

### Crate-to-framework mapping

- accessibility-sys: Accessibility (AXUIElement, AXObserver, roles/attributes, notifications)
- objc2, objc2-app-kit, objc2-foundation: AppKit/Foundation bridged ObjC types (e.g., `NSWorkspace`, `NSRunningApplication`, `NSEvent`)
- objc2-core-foundation: CoreFoundation (CFRunLoop, CFArray, CFDictionary, CFString, CFBoolean, CFNumber, CFData, CFUUID)
- objc2-core-graphics: CoreGraphics (CGDisplay, CGEventTap, CGEvent, CGError, geometry)
- notify (feature macos_fsevent): FSEvents-backed file watching (configuration reloads)
- Private SkyLight (linked as framework in `src/skylight.rs`): CGS/SLS window server, Spaces, focus/activation, cursor APIs
- Carbon (manual externs): Event Manager (process lifecycle), Process Manager, Text Input Services/Unicode utilities (keyboard mapping)

## Files and APIs

### src/skylight.rs

- Private SkyLight (CoreGraphics Services) externs:
  - Connection/process/window APIs: `SLSMainConnectionID`, `SLSGetConnectionIDForPSN`, `_SLPSGetFrontProcess`, `_SLPSSetFrontProcessWithOptions`, `SLPSPostEventRecordTo`
  - Window geometry/display mapping: `SLSGetWindowBounds`, `SLSCopyManagedDisplayForWindow`, `SLSCopyBestManagedDisplayForRect`
  - Spaces/display queries: `SLSManagedDisplayGetCurrentSpace`, `SLSCopyActiveMenuBarDisplayIdentifier`, `SLSCopyManagedDisplaySpaces`, `SLSCopyWindowsWithOptionsAndTags`
  - Window queries/iterators: `SLSWindowQueryWindows`, `SLSWindowQueryResultCopyWindows`, `SLSWindowIteratorGetCount/Advance/GetParentID/GetWindowID/GetTags/GetAttributes`
  - Hit testing/cursor: `SLSFindWindowAndOwner`, `SLSGetCurrentCursorLocation`
  - Space management mode: `SLSGetSpaceManagementMode`
- CoreGraphics/CoreFoundation display IDs: `CGDisplayCreateUUIDFromDisplayID`, `CGDisplayGetDisplayIDFromUUID`
- Accessibility bridging (private + public): `_AXUIElementGetWindow`, `AXUIElementCopyAttributeValue`, `AXUIElementSetAttributeValue`, `AXUIElementPerformAction`, `_AXUIElementCreateWithRemoteToken`
- Purpose: Single source of truth for all C/ObjC externs to SkyLight/AX used elsewhere.

### src/platform.rs

- AppKit/Foundation:
  - App init and notifications: `NSApplicationLoad`, `NSWorkspace` notifications (active display/space change, hide/unhide, wake), `NSNotificationCenter`, `NSDistributedNotificationCenter`, `NSString`, `NSDictionary`, `NSNumber`
  - Running apps and gestures: `NSRunningApplication`, `NSEvent`, `NSEventType::Gesture`, `NSTouch`, `NSTouchPhase`
- CoreGraphics:
  - Event taps: `CGEventTapCreate/Enable`, `CGEventTapLocation`, `CGEventTapPlacement`, `CGEventTapOptions`, `CGEventType`, `CGEvent`, `CGEventFlags`, `CGEventGetLocation`, `CGEventGetIntegerValueField`, `CGEventGetFlags`
  - Display callbacks: `CGDisplayRegisterReconfigurationCallback`, `CGDisplayRemoveReconfigurationCallback`, `CGDisplayChangeSummaryFlags`, `CGError`
- CoreFoundation RunLoop:
  - `CFMachPortCreateRunLoopSource`, `CFMachPortInvalidate`, `CFRunLoopAddSource`, `CFRunLoopRemoveSource`, `CFRunLoopGetMain`, `CFRunLoopRunInMode`, `kCFRunLoopDefaultMode`, `kCFRunLoopCommonModes`
- Accessibility:
  - Observers for Dock (Mission Control): `AXObserverCreate`, `AXObserverAddNotification`, `AXObserverRemoveNotification`
- Carbon (Event Manager):
  - Process lifecycle events: `GetApplicationEventTarget`, `InstallEventHandler`, `RemoveEventHandler`, `GetEventParameter`, `GetEventKind`, `GetNextProcess`
- Purpose: Glue for input (CGEventTap), workspace/display notifications, Mission Control AX notifications, and process lifecycle events; pumps them into the app via CFRunLoop.

### src/process.rs

- Carbon (Process Manager): `CopyProcessName`, `GetProcessPID`
- AppKit (process representation + KVO): `NSRunningApplication` (activation policy, finishedLaunching), KVO via `addObserver_forKeyPath_options_context` / `removeObserver_forKeyPath_context`
- Foundation/CoreFoundation: `NSString`, bridging CFString names via `CFRetained<CFString>`
- Purpose: Per-process state, readiness evaluation (finished launching + activationPolicy), and KVO hooks.

### src/app.rs

- Accessibility (AX):
  - App element and observers: `AXUIElementCreateApplication`, `AXObserverCreate`, `AXObserverAddNotification`, `AXObserverRemoveNotification`
  - Attributes/notifications: `kAXCreatedNotification`, `kAXFocusedWindowChangedNotification`, `kAXWindowsAttribute`, `kAXMainWindowAttribute`, window-level notifications (`kAXWindowMovedNotification`, etc.)
- CoreFoundation RunLoop: integrates AXObserver RunLoop source via `kCFRunLoopCommonModes` (added/removed in `util.rs`)
- Private SkyLight: `SLSGetConnectionIDForPSN`, `_SLPSGetFrontProcess`
- Purpose: AX observer lifecycle for an application; resolve focused/main windows; bridge AX events to internal events.

### src/windows.rs

- Accessibility (AX): `AXUIElementCopyAttributeValue`, `AXUIElementSetAttributeValue`, `AXUIElementPerformAction`, `AXValueCreate`, `AXValueGetValue`, attributes (`kAXPositionAttribute`, `kAXSizeAttribute`, roles/subroles, minimized)
- CoreFoundation: `CFString`, `CFArray`, `CFBoolean`, `CFNumber`, `CFEqual`, `CFUUIDCreateFromString`, `CFUUIDCreateString`
- CoreGraphics: `CGGetActiveDisplayList`, `CGDisplayBounds`, `CGRectContainsPoint`, `CGRectEqualToRect`, `CGWarpMouseCursorPosition`, `CGError`
- Private SkyLight:
  - Focus/activation: `_SLPSSetFrontProcessWithOptions`, `SLPSPostEventRecordTo`
  - Spaces/display mapping: `SLSCopyManagedDisplayForWindow`, `SLSCopyBestManagedDisplayForRect`, `SLSManagedDisplayGetCurrentSpace`, `SLSCopyActiveMenuBarDisplayIdentifier`
  - Window geometry/lookup: `SLSGetWindowBounds`, `_AXUIElementGetWindow`
  - Cursor position: `SLSGetCurrentCursorLocation`
- Purpose: Window model and operations (focus/raise, reposition/resize via AX); display/space discovery and per-display panes; cursor centering and window exposure.

### src/manager.rs

- Private SkyLight (Window Server/Spaces):
  - Global window queries: `SLSCopyWindowsWithOptionsAndTags`, `SLSWindowQueryWindows`, `SLSWindowQueryResultCopyWindows`, iterators (`GetCount/Advance/GetParentID/GetWindowID/GetTags/GetAttributes`)
  - Focused process: `_SLPSGetFrontProcess`
  - AX window token (private): `_AXUIElementCreateWithRemoteToken` (to resolve windows on inactive Spaces)
- CoreFoundation: `CFDataCreateMutable`, `CFDataIncreaseLength`, `CFDataGetMutableBytePtr`, `CFArrayGetCount`, `CFNumberType`
- Purpose: Window enumeration/filtering, creating `AXUIElementRef` via remote token brute-force, window layout/reshuffle across Spaces/displays.

### src/events.rs

- Private SkyLight: `SLSMainConnectionID`, `SLSFindWindowAndOwner`, `SLSCopyAssociatedWindows`
- CoreFoundation: `CFRetained`, `CFNumberGetValue`, `CFNumberType::SInt32Type`
- CoreGraphics: `CGEventFlags`
- Purpose: Central event types and dispatcher; hit-testing windows via SLS and associating child windows; command routing and key/mouse integration.

### src/util.rs

- Accessibility/CoreFoundation bridging:
  - AX attribute fetch: `AXUIElementCopyAttributeValue`
  - CF containers: `CFArrayCreate/GetCount/GetValueAtIndex`, `CFDictionaryGetValue`, `CFNumberCreate`, `CFString`, `kCFTypeArrayCallBacks`
- CoreFoundation RunLoop: `AXObserverGetRunLoopSource`, `CFRunLoopGetMain`, `CFRunLoopAddSource`, `CFRunLoopSourceInvalidate`
- Purpose: Safe wrappers for AX/CF pointers, run loop registration helpers, CFArray/CFDictionary utilities.

### src/config.rs

- CoreFoundation: `CFStringCreateWithCharacters`, `CFData`
- Carbon (Text Input Services/Unicode utilities):
  - Keyboard layout/key translation: `TISCopyCurrentASCIICapableKeyboardLayoutInputSource`, `TISGetInputSourceProperty`, `UCKeyTranslate`, `LMGetKbdType`, `kTISPropertyUnicodeKeyLayoutData`
- Purpose: Load config TOML; map human-friendly key specs to hardware keycodes via Carbon APIs; watch config via FSEvents (notify).

### src/main.rs

- Private SkyLight: `SLSMainConnectionID`, `SLSGetSpaceManagementMode` (guard for “Displays have separate Spaces”)
- Purpose: Entrypoint, socket command bridge, environment checks, start platform/event loops.

## Private API usage notes

- SkyLight `_SLPSSetFrontProcessWithOptions`, `SLPSPostEventRecordTo`, `_SLPSGetFrontProcess`: programmatic focus/activation and event posting; may change across macOS versions.
- `_AXUIElementCreateWithRemoteToken`: constructs `AXUIElementRef` for windows on inactive Spaces by forging remote tokens; relies on private behavior.
- `SLS*` functions (CGS/WindowServer): Spaces/display/window enumeration and metadata; undocumented/private and subject to change.
- Carbon Event Manager and Process Manager calls are legacy but still available; prefer AppKit `NSRunningApplication` where possible.
