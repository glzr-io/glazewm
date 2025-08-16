## Project Overview

GlazeWM is a window manager written in Rust, organized as a workspace with multiple crates.

### Crate Organization

- `wm` (bin): Main application, which implements the core window management logic.
  - Gets installed to `C:\Program Files\glzr.io\glazewm.exe`.
- `wm-cli` (bin/lib): CLI for interacting with the main application.
  - Gets installed to `C:\Program Files\glzr.io\cli\glazewm.exe`. This is added to `$PATH` by default.
- `wm-common` (lib): Shared types, utilities, and constants used across other crates.
- `wm-ipc-client` (lib): WebSocket client library for IPC with the main application.
- `wm-platform` (lib): Wrappers over platform-specific API's - other crates don't interact directly with the Windows and macOS API's.
- `wm-watcher` (bin): Watchdog process that ensures proper cleanup when the main application exits.
  - Gets installed to `C:\Program Files\glzr.io\glazewm-watcher.exe`.

## Rust General Guidelines

### Code Style & Formatting

- Use the project's rustfmt.toml configuration:
  - Maximum line width of 75 characters.
  - Use field init shorthand when possible.
  - Wrap comments to fit line width.
- Follow clippy suggestions unless there's a compelling reason not to.
- Use rust-analyzer with clippy for continuous linting.
- The project uses the nightly Rust toolchain. However, only use nightly features when they provide clear benefit.
- Avoid `.unwrap()` wherever possible!

## Project-Specific Conventions

### Documentation

- Document public APIs with rustdoc comments, especially important for the `wm-platform` crate.
- Rustdoc comments should include (in the following order):
  - (required) A _concise_ summary of the purpose of the function or type.
  - (required) Any notable caveats when using the function or type. Again, should be kept brief.
  - (optional) If unclear from the summary, include an additional note about the return type of the function (e.g. "Returns a vector of `NativeMonitor` sorted from left-to-right.").
  - (optional) Cases where the function might panic (use "# Panics" as heading).
  - (optional) Platform-specific notes (use "# Platform-specific" as heading).
  - (optional) Example usage, not needed for most cases (use "# Examples" as heading).
- Use punctuation marks at the end of doc comments and in-line comments.
- If using unsafe features, add a "SAFETY: ..." comment.

### Testing

- Use `#[cfg(test)]` for test modules.
- Write unit tests for core functionality.

### Module Structure

- Use `mod.rs` files for module organization
- Declare submodules with `pub mod module_name;`
- Re-export public APIs using `pub use module_name::*;` pattern
- Group related functionality in dedicated modules (e.g., commands/, events/, models/)

### Error Handling

- `anyhow` is being replaced with `thiserror` in the `wm-platform` crate. Please use `thiserror` for error handling within this crate.
- Use `thiserror` for custom error types with `#[derive(Debug, thiserror::Error)]`

### Logging & Tracing

- Use `tracing` crate for structured logging
- Log levels: `error!`, `warn!`, `info!`, `debug!`

### Platform Abstraction

- Isolate platform-specific code in `wm-platform` crate
- Use conditional compilation: `#[cfg(target_os = "windows")]`, `#[cfg(target_os = "macos")]`
- Provide unified APIs that abstract platform differences
- Handle platform-specific errors appropriately
