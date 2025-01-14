# Contributing to GlazeWM

Thanks for your interest in improving GlazeWM 💛

There are fundamentally three ways to contribute:

1. **Opening issues**: If you believe you've found a bug or have a feature request, open an issue to discuss it.

2. **Helping triage issues**: Add supporting details and suggestions to existing issues.

3. **Submitting PRs**: Submit a PR that fixes a bug or implements a feature.

The [#glazewm-dev channel ⚡](https://discord.com/invite/ud6z3qjRvM) is also available for any concerns not covered in this guide, please join us!

## Pull requests & dev workflow

For PRs, a good place to start are the issues marked as [`good first issue`](https://github.com/glzr-io/glazewm/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) or [`help wanted`](https://github.com/glzr-io/glazewm/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22). PR's don't have a requirement to have a corresponding issue, but if there is one already, please drop a comment in the issue and we can assign it to you.

### Setup

First fork, then clone the repo:

```shell
git clone git@github.com:your-username/glazewm.git
```

If not already installed, [install Rust](https://rustup.rs/), then run:

```shell
# `cargo build` will build all binaries and libraries.
# `cargo run` will run the default binary, which is configured to be the wm.
cargo build && cargo run
```

After making your changes, push to your fork and [submit a pull request](https://github.com/glzr-io/zebar/pulls) against the `main` branch. Please try to address only a single feature or fix in the PR so that it's easy to review.

### Tips

If using VSCode, it's recommended to use the [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension. Get automatic linting by adding this to your VSCode's `settings.json`:

```json
{
  "rust-analyzer.check.command": "clippy"
}
```

## Codebase overview

Knowledge of the entire codebase should never be required to make changes. The following should hopefully help with understanding a particular part of the codebase.

### Crates

GlazeWM is organized into several Rust crates:

- `wm` (bin): Main application, which implements the core window management logic.
  - Gets installed to `C:\Program Files\glzr.io\glazewm.exe`.
- `wm-cli` (bin/lib): CLI for interacting with the main application.
  - Gets installed to `C:\Program Files\glzr.io\cli\glazewm.exe`. This is added to `$PATH` by default.
- `wm-common` (lib): Shared types, utilities, and constants used across other crates.
- `wm-ipc-client` (lib): WebSocket client library for IPC with the main application.
- `wm-platform` (lib): Wrappers over Windows APIs - other crates don't interact directly with the Windows APIs.
- `wm-watcher` (bin): Watchdog process that ensures proper cleanup when the main application exits.
  - Gets installed to `C:\Program Files\glzr.io\glazewm-watcher.exe`.

### Commands & events

GlazeWM uses a command-event architecture. The state of the WM (stored in [`WmState`](https://github.com/glzr-io/glazewm/blob/main/packages/wm/src/wm_state.rs)) is modified via [commands](https://github.com/glzr-io/glazewm/tree/main/packages/wm/src/commands) and [events](https://github.com/glzr-io/glazewm/tree/main/packages/wm/src/events).

- Commands are run as a result of keybindings, IPC calls, the CLI (which calls IPC internally), or by being called from another command. Most commands are just for internal use and might not have a public-facing API.
- Events arise from the Windows platform (e.g. a window being created, destroyed, focused, etc.). Each of these events have a handler that then modifies the WM state.

Commands and events are processed in a loop in [`start_wm`](https://github.com/glzr-io/glazewm/blob/main/packages/wm/src/main.rs#L68).

## Container tree

Windows in GlazeWM are organized within a tree hierarchy with the following "container" types:

- Root
- Monitors (physical displays)
- Workspaces (virtual groups of windows)
- Split containers (for tiling layouts)
- Windows (application windows)

Here's an example container tree:

```
                                Root
                                  |
                 +----------------+--------------+
                 |                               |
             Monitor 1                       Monitor 2
                 |                               |
        +--------+------+                        |
        |               |                    Workspace 1
    Workspace 1     Workspace 2                  |
        |               |                  Split Container
        |               |                 (vertical layout)
   +----|----+      Window                       |
   |         |     (Spotify)              +------+------+
   |     Window                           |             |
   |   (Terminal                      Window         Window
   |   floating)                     (Discord)       (Slack)
   |
Split Container
(horizontal layout)
   |
   +-------------+
   |             |
Window         Window
(Chrome)     (VS Code)
```

Windows can be either tiling (nested within split containers) or non-tiling (floating, minimized, maximized, or fullscreen). Non-tiling windows are always direct children of a workspace. Split containers can only have windows as children, and must have at least one child window.
