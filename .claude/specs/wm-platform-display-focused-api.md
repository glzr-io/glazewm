# WM-Platform Display API - AI-Focused Implementation Plan

## Task Overview

Create production-ready display enumeration and management API for wm-platform crate.

## Critical Path Tasks

### Phase 1: Core Error Handling

**File:** `packages/wm-platform/src/error.rs`
**Missing:** `DisplayNotFound`, `PrimaryDisplayNotFound`, `DisplayDeviceNotFound` error variants
**Action:** Add these specific error variants with `#[error("...")]` attributes

### Phase 2: Windows Implementation Fixes

**File:** `packages/wm-platform/src/platform_impl/windows/display.rs`

#### Task 2.1: `get_monitor_info()` method

```rust
fn get_monitor_info(&self) -> Result<MONITORINFOEXW> {
    // Use GetMonitorInfoW with self.monitor_handle
    // Return populated MONITORINFOEXW struct
}
```

#### Task 2.2: `get_device_name()` method

```rust
fn get_device_name(&self) -> Result<String> {
    // Call get_monitor_info(), extract szDevice field
    // Convert from UTF-16 to String
}
```

#### Task 2.3: `output_technology()` method

```rust
pub fn output_technology(&self) -> Result<OutputTechnology> {
    // Parse hardware_id field for technology keywords
    // Return appropriate OutputTechnology enum variant
}
```

#### Task 2.4: `is_builtin()` method

```rust
pub fn is_builtin(&self) -> Result<bool> {
    // Check hardware_id for "Internal", "Laptop", "Built" keywords
    // Return boolean result
}
```

### Phase 3: macOS Implementation Fixes

**File:** `packages/wm-platform/src/platform_impl/macos/display.rs`

#### Task 3.1: `Display::id()` method

```rust
pub fn id(&self) -> DisplayId {
    DisplayId(self.cg_display_id)
}
```

#### Task 3.2: `is_builtin()` for DisplayExtMacos

```rust
fn is_builtin(&self) -> Result<bool> {
    // Use CGDisplayIsBuiltin(self.cg_display_id)
    // Return boolean result
}
```

#### Task 3.3: Module-level functions

- `all_display_devices()` - Use CGGetOnlineDisplayList
- `active_display_devices()` - Use CGGetActiveDisplayList
- `display_from_point()` - Use CGDisplayAtPoint
- `primary_display()` - Use CGMainDisplayID

### Phase 4: Public API Surface

**File:** `packages/wm-platform/src/display.rs`

#### Task 4.1: Top-level convenience functions

```rust
pub fn all_displays() -> Result<Vec<Display>>
pub fn all_display_devices() -> Result<Vec<DisplayDevice>>
pub fn primary_display() -> Result<Display>
pub fn display_from_point(point: Point) -> Result<Display>
```

#### Task 4.2: Implement PartialEq/Eq traits

```rust
impl PartialEq for Display {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
```

### Phase 5: Platform Hook Integration

**File:** `packages/wm-platform/src/platform_hook.rs`
**Action:** Remove async from display functions, fix function signatures to match implementations

## Implementation Specifications

### Error Types Required

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Display not found")]
    DisplayNotFound,

    #[error("Primary display not found")]
    PrimaryDisplayNotFound,

    #[error("Display device not found")]
    DisplayDeviceNotFound,

    #[error("Display enumeration failed: {0}")]
    DisplayEnumerationFailed(String),
}
```

### Windows API Integration Points

- `GetMonitorInfoW` for monitor information
- `EnumDisplayDevicesW` for device enumeration
- `EnumDisplaySettingsW` for display modes
- `MonitorFromPoint` for point-to-display mapping

### macOS API Integration Points

- `CGGetActiveDisplayList` for display enumeration
- `CGDisplayIsBuiltin` for builtin detection
- `CGDisplayAtPoint` for point-to-display mapping
- `CGMainDisplayID` for primary display

### OutputTechnology Enum Extensions

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputTechnology {
    Internal,
    HDMI,
    DisplayPort,
    DVI,
    VGA,
    USB,
    Thunderbolt,
    Unknown,
}
```

## Testing Requirements

### Unit Tests (per platform)

- `test_all_displays()` - Verify non-empty results
- `test_primary_display()` - Verify primary detection
- `test_display_devices()` - Verify device enumeration
- `test_display_equality()` - Verify ID-based equality

### Integration Tests

- Display hot-plug scenarios
- Multi-monitor configurations
- Point-to-display mapping accuracy

## Success Criteria

1. All compilation errors resolved
2. All `todo!()` and TODO comments replaced with working code
3. Basic display enumeration works on both platforms
4. Primary display detection works
5. Point-to-display mapping works
6. Unit tests pass on both platforms

## File Change Summary

- `error.rs` - Add 4 new error variants
- `windows/display.rs` - Implement 4 missing methods
- `macos/display.rs` - Implement 6 missing methods/functions
- `display.rs` - Add 4 public functions, implement PartialEq/Eq
- `lib.rs` - Export new public functions

This plan focuses on concrete, implementable tasks with specific function signatures and clear success criteria for each phase.
