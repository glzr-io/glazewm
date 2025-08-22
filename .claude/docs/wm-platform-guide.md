# wm-platform Development Guide

This guide provides technical details and best practices for working with the `wm-platform` crate, which provides cross-platform abstractions for window management APIs.

## Architecture Overview

The `wm-platform` crate follows these patterns:

- `platform_impl/` for concrete platform implementations

## Reference Material

For platform-specific API documentation, use the Context7 MCP server:

- **Windows (`windows` crate)**: Use `/microsoft/windows-rs` and `/websites/docs_rs-windows-latest-windows`
- **macOS (`objc2` crate ecosystem)**: Use crate names like "objc2-foundation", "objc2-app-kit", etc.
  - Foundation types: `/websites/docs_rs-objc2-foundation-latest-objc2_foundation`
  - Use topic parameters for specific types (e.g., "NSString", "NSArray", "memory management")

For referencing similar implementations in other related projects:

- See the summaries available in `.claude/references/`:
  - **Windows**: komorebi.md.
  - **macOS**: glide-wm.md, paneru.md
- You can use the GitHub MCP server to look up specific files from these references.

## Error Handling

- Use `thiserror` for custom error types.
- Avoid `.unwrap()` wherever possible
- Handle platform-specific error codes properly

## Platform API Integration

### Windows Integration

- Use the `windows` crate for Win32 API access (NOT `winapi`)
- Handle HRESULT return codes properly
- Manage COM object lifetimes correctly
- Convert between Rust and Windows types safely

### macOS Integration

- Use `objc2` ecosystem crates for Cocoa/Foundation APIs
- Handle Objective-C memory management with `Retained<T>`
- Convert between Rust and NSString/CFString properly
- Use Core Foundation types when necessary
- Handle AXUIElement accessibility APIs correctly

## Important Crate Migration Notes

When studying reference projects, be aware of crate ecosystem differences:

### Windows Crate Differences

- Reference projects may use the older `winapi` crate
- GlazeWM uses the modern `windows` crate (official Microsoft-maintained)
- API signatures and return types differ between these crates
- The `windows` crate provides better type safety and more idiomatic Rust patterns

### macOS Crate Differences

- Reference projects may use older crates: `core-foundation`, `core-foundation-sys`, `core-graphics`, `accessibility`, `accessibility-sys`
- GlazeWM uses the modern `objc2` ecosystem crates
- Different memory management patterns (manual retain/release vs `Retained<T>`)
- Different API surfaces and safety guarantees

## Safety Requirements

- Validate all raw pointers before dereferencing
- Ensure proper cleanup of native resources
- Handle platform API call errors gracefully
- Use Rust ownership system to prevent resource leaks
- Add "SAFETY: ..." comments for any unsafe code
