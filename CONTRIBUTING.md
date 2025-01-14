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

Knowledge of the entire codebase is rarely required to make changes. Components are generally well isolated and don't have a lot of interdependencies.

GlazeWM is organized into several Rust crates:

- `wm` (bin): Main application, which implements the core window management logic.
  - Distributed as `glazewm.exe`.
- `wm-cli` (bin/lib): CLI for interacting with the main application.
  - Distributed as `cli/glazewm.exe`. This is added to `$PATH` by default.
- `wm-common` (lib): Shared types, utilities, and constants used across other crates.
- `wm-ipc-client` (lib): WebSocket client library for IPC with the main application.
- `wm-platform` (lib): Abstractions over Windows APIs so that other crates don't interact directly with the Windows APIs.
- `wm-watcher` (bin): Watchdog process that ensures proper cleanup when the main application exits.
  - Distributed as `glazewm-watcher.exe`.

### Key concepts

GlazeWM uses a command-event architecture.

- Commands can come from keybindings, IPC calls, or the CLI (which calls IPC internally). Commands are simply functions that modify the state of the WM.
- Events are sent from the main application to the CLI via WebSockets.
