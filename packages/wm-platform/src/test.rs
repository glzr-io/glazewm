#[macro_use]
extern crate libtest_mimic_collect;

mod dispatcher;
mod display;
mod error;
mod event_loop;
mod event_loop_installer;
mod keybinding_listener;
mod models;
mod mouse_listener;
mod native_window;
mod platform_event;
mod platform_impl;
mod window_listener;

pub use dispatcher::*;
pub use display::*;
pub use error::*;
pub use event_loop::*;
pub use event_loop_installer::*;
pub use keybinding_listener::*;
pub use models::*;
pub use mouse_listener::*;
pub use native_window::*;
pub use platform_event::*;
#[cfg(target_os = "macos")]
pub use platform_impl::{
  DisplayDeviceExtMacOs, DisplayExtMacOs, NativeWindowExtMacOs,
};
#[cfg(target_os = "windows")]
pub use platform_impl::{DisplayDeviceExtWindows, DisplayExtWindows};
pub use window_listener::*;

pub fn main() {
  // Due to macOS requiring the main thread for some UI APIs, these
  // tests must execute on the main thread. Until this is natively
  // supported via cargo's test harness, we use `libtest_mimic_collect`.
  //
  // To run these tests, run `cargo test <...args> -- --test-threads=1`.
  //
  // Ref: https://github.com/rust-lang/rust/issues/104053
  libtest_mimic_collect::TestCollection::run();
}
