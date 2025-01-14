# Contributing to GlazeWM

PRs are always a huge help 💛. Check out issues marked as [good first issue](https://github.com/glzr-io/glazewm/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) or [help wanted](https://github.com/glzr-io/glazewm/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22) to get started.

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

After making your changes, push to your fork and [submit a pull request](https://github.com/glzr-io/zebar/pulls). Please try to address only a single feature or fix in the PR so that it's easy to review.

## Dev workflow

If using VSCode, it's recommended to use the [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) extension. Get automatic linting by adding this to your VSCode's `settings.json`:

```json
{
  "rust-analyzer.check.command": "clippy"
}
```

## Project architecture

GlazeWM is organized into several Rust crates:

- `wm` (bin): Main application, which implements the core window management logic.
- `wm-cli` (bin/lib): CLI for interacting with the main application.
- `wm-common` (lib): Shared types, utilities, and constants used across other crates.
- `wm-ipc-client` (lib): WebSocket client library for IPC with the main application.
- `wm-platform` (lib): Abstractions over Windows APIs so that other crates don't interact directly with the Windows APIs.
- `wm-watcher` (bin): Watchdog process that ensures proper cleanup when the main application exits.

### Key concepts

GlazeWM uses a command-event architecture.

- Commands can come from keybindings, IPC calls, or the CLI (which calls IPC internally). Commands are simply functions that modify the state of the WM.
- Events are sent from the main application to the CLI via WebSockets.

## Need help?

- Join community discussions on [Discord](https://discord.com/invite/ud6z3qjRvM).
- Open an issue for bugs or feature requests.
- Check existing issues and pull requests.

Contributor discussions take place in the `#glazewm-dev` channel on the Discord server. Feel free to ask any dev-related questions there or reach out to me personally (`@lars.be`).
