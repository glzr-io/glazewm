# Build

Build from the repository root.

## Requirements

- Rust nightly.
- Native toolchain for your platform.
  - Windows: MSVC toolchain.
  - macOS: Xcode command line tools.

This repo pins Rust nightly via `rust-toolchain.toml`.

## Local dev build

Installs the pinned nightly automatically if needed.

```powershell
cargo build
```

Run the main WM binary:

```powershell
cargo run
```

## What root `cargo build` builds

The workspace `default-members` are:

- `packages/wm`
- `packages/wm-cli`

So root `cargo build` builds:

- `glazewm`
- `glazewm-cli`

It does not build `wm-watcher`.

## Full workspace build

On Windows, use `--workspace` to include `wm-watcher`:

```powershell
cargo build --workspace
```

Release build:

```powershell
cargo build --locked --release --workspace
```

## CI-equivalent build commands

Windows x64:

```powershell
cargo build --locked --release --target x86_64-pc-windows-msvc --workspace
```

Windows x64 with UIAccess manifest enabled:

```powershell
cargo build --locked --release --target x86_64-pc-windows-msvc --workspace --features ui_access
```

Windows ARM64:

```powershell
cargo build --locked --release --target aarch64-pc-windows-msvc --workspace
```

macOS Intel:

```powershell
cargo build --locked --release --target x86_64-apple-darwin
```

macOS Apple Silicon:

```powershell
cargo build --locked --release --target aarch64-apple-darwin
```

## Version number

Build metadata uses the `VERSION_NUMBER` environment variable. It defaults to `0.0.0` via `.cargo/config.toml`.

Example:

```powershell
$env:VERSION_NUMBER="1.2.3"
cargo build --locked --release --workspace
```

## Build outputs

Native-target release binaries are written under:

```text
target/<target-triple>/release/
```

Examples:

- `target/x86_64-pc-windows-msvc/release/glazewm.exe`
- `target/x86_64-pc-windows-msvc/release/glazewm-cli.exe`
- `target/x86_64-pc-windows-msvc/release/glazewm-watcher.exe`
- `target/aarch64-apple-darwin/release/glazewm`

## Packaging

### Windows installers

CI packaging installs:

- WiX 5.
- WiX extensions:
  - `WixToolset.UI.wixext`
  - `WixToolset.Util.wixext`
  - `WixToolset.BootstrapperApplications.wixext`
- `AzureSignTool`

Then it runs:

```powershell
./resources/scripts/package.ps1 -VersionNumber $env:VERSION
```

### macOS installer

CI builds both macOS targets, combines binaries with `lipo`, then signs and notarizes the DMG.

## Notes

- `cargo build` from the root is correct for local development.
- If you need all binaries on Windows, prefer `cargo build --workspace`.
- If you want reproducible CI-style builds, use `--locked --release` and an explicit `--target`.
