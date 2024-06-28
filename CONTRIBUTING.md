# Contributing to GlazeWM

### Installing Rust

Rust **nightly** is currently used. [`rustup`](https://rustup.rs/) is the recommended way to set up the Rust toolchain.

### Development

To start the project in development mode:

```shell
# `cargo build` will build both the watcher + wm binaries.
# `cargo run` will run the default binary, which is configured to be the wm.
cargo build && cargo run
```
