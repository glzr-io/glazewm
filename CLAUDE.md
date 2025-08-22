## Project overview

GlazeWM is a window manager written in Rust. The project is organized as a Cargo workspace with multiple crates.

### Crates overview

- `wm` (bin): Main application, which implements the core window management logic.
  - Gets installed to `C:\Program Files\glzr.io\glazewm.exe`.
- `wm-cli` (bin/lib): CLI for interacting with the main application.
  - Gets installed to `C:\Program Files\glzr.io\cli\glazewm.exe`. This is added to `$PATH` by default.
- `wm-common` (lib): Shared types, utilities, and constants used across other crates.
- `wm-ipc-client` (lib): WebSocket client library for IPC with the main application.
- `wm-platform` (lib): Wrappers over platform-specific API's - other crates don't interact directly with the Windows and macOS API's.
  - See `.claude/doc/wm-platform-guide.md` for development guidelines and best practices when working with the `wm-platform` crate.
- `wm-watcher` (bin): Watchdog process that ensures proper cleanup when the main application exits.
  - Gets installed to `C:\Program Files\glzr.io\glazewm-watcher.exe`.

### Code style & formatting

- Do not leave partial or simplified implementations!
- Avoid `.unwrap()` wherever possible!
- Follow clippy suggestions unless there's a compelling reason not to.
- Use rust-analyzer with clippy for continuous linting.
- The project uses the nightly Rust toolchain. However, only use nightly features when they provide clear benefit.

### Code comments

- Document public APIs with rustdoc comments, especially important for the `wm-platform` crate.
- Rustdoc comments should include (in the following order):
  - (required) A _concise_ summary of the purpose of the function or type.
  - (required) Any notable caveats when using the function or type. Again, should be kept brief.
  - (optional) If unclear from the summary, include an additional note about the return type of the function (e.g. "Returns a vector of `NativeMonitor` sorted from left-to-right.").
  - (optional) Cases where the function might panic (use "# Panics" as heading).
  - (optional) Platform-specific notes (use "# Platform-specific" as heading).
  - (optional) Example usage, not needed for most cases (use "# Examples" as heading).
- Use punctuation marks at the end of doc comments and in-line comments.
- Wrap names of types in \` (e.g. `ExampleStruct`).
- If using unsafe features, add a "SAFETY: ..." comment.

### Testing

- Use `#[cfg(test)]` for test modules.
- Write unit tests for core functionality.

### Error handling

- `anyhow` is being replaced with `thiserror` in the `wm-platform` crate. Please use `thiserror` for error handling within this crate.
- Use `thiserror` for custom error types with `#[derive(Debug, thiserror::Error)]`.

### Logging

- Use `tracing` crate for logging.
- Log levels: `error!`, `warn!`, `info!`, `debug!`.
