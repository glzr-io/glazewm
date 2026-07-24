<project_overview>
GlazeWM is a window manager for macOS and Windows, written in Rust.

Crate structure:

- **wm** (bin): Main application, which implements the core window management logic. Install path on Windows: `C:\Program Files\glzr.io\glazewm.exe`
- **wm-cli** (bin, lib): CLI for interacting with the main application. Added to `$PATH` by default. Install path on Windows: `C:\Program Files\glzr.io\cli\glazewm.exe`
- **wm-common** (lib): Shared types, utilities, and constants used across other crates.
- **wm-ipc-client** (lib): WebSocket client library for IPC with the main application.
- **wm-platform** (lib): Wrappers over platform-specific APIs; other crates do not call Windows/macOS APIs directly.
- **wm-watcher** (Windows-only) (bin): Watchdog process ensuring proper cleanup when the main application exits. Install path on Windows: `C:\Program Files\glzr.io\glazewm-watcher.exe`

</project_overview>

<output_guidelines>

- Be extremely concise. Sacrifice grammar for the sake of conciseness.
- Do not leave partial or simplified implementations.
- The required quality standard is high. Low quality code will be rejected.
- Do not proceed with solutions that are hacky. Solutions must be robust, maintainable, and extendable. Ask guiding questions if uncertain about a solution.

</output_guidelines>

<code_style_guidelines>

- Avoid `.unwrap()` wherever possible.
- For error handling:
  - Use `crate::Error` and `crate::Result` within the `wm-platform` crate.
  - Use `anyhow` in all other crates.
- For logging, use `tracing` macros (e.g. `tracing::info!("...")`).

</code_style_guidelines>

<code_comment_guidelines>

- Functions should always be documented.
- Use punctuation mark at the end of all comments.
- If using unsafe features, include a "SAFETY: ..." comment.
- Wrap type names in backticks (e.g. `NativeMonitor`).

Comment structure:

```rs
/// <Concise summary of the function or type>
///
/// (optional) <Notable caveats for usage (kept brief)>
///
/// (optional) <Describe return value if ambiguous (e.g. "Returns a vector of `NativeMonitor`, sorted by their position from left-to-right.")>
///
/// (optional) # Example usage
///
/// <Code block with example usage>
///
/// (optional) # Platform-specific
///
/// <Bullet-point list of behavioral differences on macOS vs Windows>
pub fn my_function() { ... }
```

</code_comment_guidelines>

<test_guidelines>

- Use `#[cfg(test)]` for test modules.
- Write unit tests for core functionality.

</test_guidelines>
