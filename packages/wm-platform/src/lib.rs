#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![feature(iterator_try_collect)]
#![feature(once_cell_try)]

mod dispatcher;
mod display;
mod error;
mod event_loop;
mod event_loop_installer;
mod key;
mod keybinding_listener;
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
pub use key::*;
pub use keybinding_listener::*;
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
