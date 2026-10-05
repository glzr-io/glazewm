#![feature(iterator_try_collect)]

#[macro_use]
extern crate libtest_mimic_collect;

mod autostart;
mod dispatcher;
mod display;
mod display_listener;
mod error;
mod event_loop;
mod keybinding_listener;
mod models;
mod mouse_listener;
mod native_window;
mod platform_event;
mod platform_impl;
mod single_instance;
mod thread_bound;
mod window_listener;

pub use autostart::*;
pub use dispatcher::*;
pub use display::*;
pub use display_listener::*;
pub use error::*;
pub use event_loop::*;
pub use keybinding_listener::*;
pub use models::*;
pub use mouse_listener::*;
pub use native_window::*;
pub use platform_event::*;
pub use single_instance::*;
pub use thread_bound::*;
pub use window_listener::*;

pub fn main() {
  // Some UI APIs must be called from the main thread, so these tests
  // have to execute there. Until that's natively supported via cargo's
  // test harness, we use `libtest_mimic_collect`.
  //
  // To run these tests, run `cargo test <...args> -- --test-threads=1`.
  //
  // Ref: https://github.com/rust-lang/rust/issues/104053
  libtest_mimic_collect::TestCollection::run();
}
