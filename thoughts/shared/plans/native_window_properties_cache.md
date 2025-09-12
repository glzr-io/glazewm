# NativeWindowProperties Cache Implementation Plan

## Overview

Complete the implementation of NativeWindowProperties as a performance cache for window properties, eliminating expensive native API calls by storing last known valid window properties.

## Current State Analysis

### What Exists:
- `NativeWindowProperties` struct defined in `/Users/lars/projects/glazewm/packages/wm/src/models/native_window_properties.rs:4-12`
- Window constructors accept `NativeWindowProperties` parameter but don't store it
- Event handlers exist for property changes but only invalidate native caches

### What's Missing:
- `native_properties: NativeWindowProperties` field in `TilingWindowInner` and `NonTilingWindowInner` structs
- `native_properties()` getter method implementation  
- Cache refresh logic in event handlers
- Conversion from native property access to cached property access

### Key Discoveries:
- TODO comments at `tiling_window.rs:126-138` and `non_tiling_window.rs:126-138` indicate planned refactor
- Current pattern: `window.native().title().unwrap_or_else(|_| "Error".to_string())`
- Performance issue: Every property access triggers platform-specific system calls
- Windows platform has some caching via `Memo<T>`, macOS has none

## Desired End State

After implementation:
- All window property access uses cached `NativeWindowProperties` instead of native API calls
- Cache updates individual properties when window events indicate property changes
- Performance improvement from eliminating redundant native API calls
- Consistent property access pattern across the codebase

### Verification:
- Search codebase for `native().title()`, `native().class_name()`, `native().process_name()` - should find none
- All TODO comments about cached properties should be resolved
- Property access should be fast and not trigger platform API calls

## What We're NOT Doing

- Not changing the `NativeWindowProperties` struct fields or types
- Not modifying the event system or adding new event types
- Not changing the public API exposed to CLI or IPC clients
- Not implementing property change detection (rely on existing event system)

## Implementation Approach

Implement in phases to maintain functionality while refactoring. Each phase builds on the previous and can be tested independently.

## Phase 1: Add Native Properties Storage to Window Structs

### Overview
Add the missing `native_properties` field to window inner structs and implement getter methods.

### Changes Required:

#### 1. Update TilingWindowInner Struct
**File**: `packages/wm/src/models/tiling_window.rs:33-50`
**Changes**: Add `native_properties: NativeWindowProperties` field

```rust
struct TilingWindowInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  tiling_size: f32,
  native: NativeWindow,
  native_properties: NativeWindowProperties,  // Add this field
  state: WindowState,
  // ... rest of fields unchanged
}
```

#### 2. Update NonTilingWindowInner Struct
**File**: `packages/wm/src/models/non_tiling_window.rs:28-44`
**Changes**: Add `native_properties: NativeWindowProperties` field

```rust
struct NonTilingWindowInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  native: NativeWindow,
  native_properties: NativeWindowProperties,  // Add this field
  state: WindowState,
  // ... rest of fields unchanged
}
```

#### 3. Update Window Constructors
**Files**: `packages/wm/src/models/tiling_window.rs:66-83` and `packages/wm/src/models/non_tiling_window.rs:61-78`
**Changes**: Store the properties parameter

```rust
// In both TilingWindow::new() and NonTilingWindow::new()
let window = WindowInner {
  // ... existing fields
  native_properties: properties,  // Store the passed properties
  // ... rest of fields
};
```

#### 4. Add Native Properties Methods
**File**: `packages/wm/src/traits/common_getters.rs`
**Changes**: Add getter and generic update method to CommonGetters trait and impl_common_getters! macro

```rust
// Add to CommonGetters trait
fn native_properties(&self) -> NativeWindowProperties;
fn update_native_properties<F>(&self, updater: F) 
where 
  F: FnOnce(&mut NativeWindowProperties);

// Add to impl_common_getters! macro implementation
pub fn native_properties(&self) -> NativeWindowProperties {
  self.0.borrow().native_properties.clone()
}

pub fn update_native_properties<F>(&self, updater: F) 
where 
  F: FnOnce(&mut NativeWindowProperties)
{
  updater(&mut self.0.borrow_mut().native_properties);
}
```

### Success Criteria:

#### Automated Verification:
- [x] Code compiles successfully: `cargo check`
- [x] No linting errors: `cargo clippy`
- [x] All existing tests pass: `cargo test`

#### Manual Verification:
- [x] Window constructors store native_properties correctly
- [x] `window.native_properties()` method returns expected values
- [x] No runtime panics when accessing native_properties

---

## Phase 2: Update Property Access Throughout Codebase

### Overview
Replace all direct native property access with cached property access.

### Changes Required:

#### 1. Update Window Rule Matching
**File**: `packages/wm/src/user_config.rs:228-231`
**Changes**: Use cached properties instead of native calls

```rust
// Replace current implementation
let window_title = window.native().title().unwrap_or_default();
let window_class = window.native().class_name().unwrap_or_default();
let window_process = window.native().process_name().unwrap_or_default();

// With cached property access
let window_title = window.native_properties().title;
let window_class = window.native_properties().class_name;
let window_process = window.native_properties().process_name;
```

#### 2. Update Display Formatting
**File**: `packages/wm/src/models/container.rs:219-221`
**Changes**: Use cached properties for display formatting

```rust
// Replace native calls with cached properties
let title = window.native_properties().title;
let class = window.native_properties().class_name;
let process = window.native_properties().process_name;
```

#### 3. Update Window Serialization (TilingWindow)
**File**: `packages/wm/src/models/tiling_window.rs:127-138`
**Changes**: Use cached properties and remove TODO comment

```rust
// Replace the TODO section with:
title: self.native_properties().title,
class_name: self.native_properties().class_name,
process_name: self.native_properties().process_name,
```

#### 4. Update Window Serialization (NonTilingWindow)
**File**: `packages/wm/src/models/non_tiling_window.rs:127-138`
**Changes**: Use cached properties and remove TODO comment

```rust
// Replace the TODO section with:
title: self.native_properties().title,
class_name: self.native_properties().class_name,
process_name: self.native_properties().process_name,
```

#### 5. Update State Checks
**Files**: Various event handlers and commands
**Changes**: Use cached boolean properties for state checks

```rust
// Replace patterns like:
window.native().is_visible().unwrap_or(false)
window.native().is_minimized()?
window.native().is_maximized()?

// With cached property access:
window.native_properties().is_visible
window.native_properties().is_minimized
window.native_properties().is_maximized
```

### Success Criteria:

#### Automated Verification:
- [ ] Code compiles successfully: `cargo check`
- [ ] No linting errors: `cargo clippy`
- [ ] All tests pass: `cargo test`
- [ ] Search finds no `native().title()` calls: `rg "native\(\)\.title\(\)"`
- [ ] Search finds no `native().class_name()` calls: `rg "native\(\)\.class_name\(\)"`
- [ ] Search finds no `native().process_name()` calls: `rg "native\(\)\.process_name\(\)"`

#### Manual Verification:
- [ ] Window rule matching works correctly
- [ ] Window display formatting shows correct information
- [ ] IPC/CLI clients receive correct window properties
- [ ] No performance regression in window operations

---

## Phase 3: Implement Individual Property Updates

### Overview
Update event handlers to refresh individual properties in the cache when they change, rather than doing full cache refreshes.

### Changes Required:

#### 1. Update Window Title Change Handler
**File**: `packages/wm/src/events/handle_window_title_changed.rs:20`
**Changes**: Update cached title after invalidating native title

```rust
// After existing native invalidation
try_warn!(window.native().invalidate_title());

// Update cached title using generic updater
if let Ok(new_title) = window.native().title() {
  window.update_native_properties(|props| props.title = new_title);
} else {
  warn!("Failed to refresh window title");
}
```

#### 2. Update Window Location Change Handler
**File**: `packages/wm/src/events/handle_window_location_changed.rs`
**Changes**: Update cached state properties after invalidation calls

```rust
// After existing invalidation calls
try_warn!(window.native().invalidate_is_maximized());
try_warn!(window.native().invalidate_is_minimized());

// Update cached state properties using generic updater
if let Ok(is_maximized) = window.native().is_maximized() {
  if let Ok(is_minimized) = window.native().is_minimized() {
    window.update_native_properties(|props| {
      props.is_maximized = is_maximized;
      props.is_minimized = is_minimized;
    });
  }
}
```

#### 3. Update Window State Change Handlers
**Files**: `handle_window_minimized.rs`, `handle_window_minimize_ended.rs`, `handle_window_hidden.rs`, `handle_window_shown.rs`
**Changes**: Update specific cached properties after native property invalidation

```rust
// Example for handle_window_minimized.rs:
try_warn!(window.native().invalidate_is_minimized());
if let Ok(is_minimized) = window.native().is_minimized() {
  window.update_native_properties(|props| props.is_minimized = is_minimized);
}

// Example for handle_window_hidden.rs:
if let Ok(is_visible) = window.native().is_visible() {
  window.update_native_properties(|props| props.is_visible = is_visible);
}
```

#### 4. Initial Property Population
**File**: Where windows are first created/managed
**Changes**: Ensure properties are populated when window is initially managed

### Success Criteria:

#### Automated Verification:
- [ ] Code compiles successfully: `cargo check`
- [ ] No linting errors: `cargo clippy`
- [ ] All tests pass: `cargo test`

#### Manual Verification:
- [ ] Window title changes are reflected in cached native_properties
- [ ] Window state changes (minimize, maximize) update cache correctly
- [ ] Window visibility changes update cache correctly
- [ ] Properties remain accurate after window events
- [ ] No performance regression from individual property updates

---

## Phase 4: Testing and Validation

### Overview
Comprehensive testing to ensure cache accuracy and performance improvements.

### Testing Strategy

#### Unit Tests:
**File**: `packages/wm/src/models/native_window_properties.rs`
**Tests**: Add tests for NativeWindowProperties functionality

```rust
#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn test_properties_cache_access() {
    // Test that cached properties are accessible
  }
  
  #[test] 
  fn test_properties_cache_refresh() {
    // Test that cache refresh updates properties correctly
  }
}
```

#### Integration Tests:
- Test window rule matching with cached properties
- Test window event handling with cache updates
- Test IPC serialization with cached properties

### Manual Testing Steps:

1. **Property Access Performance**:
   - Create multiple windows
   - Verify property access is fast (no visible delay)
   - Compare with previous native API call performance

2. **Cache Consistency**:
   - Change window titles externally
   - Verify cached properties update correctly
   - Minimize/maximize windows and verify state cache

3. **Window Rule Matching**:
   - Configure window rules based on title, class, process
   - Verify rules still match correctly with cached properties

4. **IPC/CLI Integration**:
   - Query window information via CLI
   - Verify returned properties match actual window state

### Success Criteria:

#### Automated Verification:
- [ ] All unit tests pass: `cargo test native_window_properties`
- [ ] Integration tests pass: `cargo test`
- [ ] No memory leaks in property caching
- [ ] Performance benchmarks show improvement

#### Manual Verification:
- [ ] Window properties display correctly in debug output
- [ ] Window rules work correctly with cached properties  
- [ ] CLI commands return accurate window information
- [ ] No stale property data after window events
- [ ] Overall system responsiveness improved

## Performance Considerations

**Expected Improvements**:
- Reduced native API calls from O(n) per property access to O(1) cache lookup
- Elimination of platform-specific system call overhead
- More consistent performance across Windows and macOS platforms

**Memory Impact**:
- Minimal increase: ~200 bytes per window for cached properties
- Trade-off justified by performance improvement for frequent property access

## Migration Notes

**Backward Compatibility**:
- Public APIs remain unchanged
- IPC protocol unchanged
- CLI output format unchanged

**Rollback Strategy**:
- Can revert property access to native calls if issues found
- Cache refresh logic can be disabled independently
- Changes are localized to window property access patterns

## References

- Current inefficient access pattern: `tiling_window.rs:126-138`, `non_tiling_window.rs:126-138`
- Event handlers needing cache updates: `handle_window_title_changed.rs:20`, `handle_window_location_changed.rs`, etc.
- NativeWindowProperties definition: `native_window_properties.rs:4-12`
- Window rule matching: `user_config.rs:228-231`