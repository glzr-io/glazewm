#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![feature(iterator_try_collect)]
#![feature(once_cell_try)]

mod display;
mod error;
mod native_window;
mod platform_event;
mod platform_hook;
mod platform_hook_installer;
mod platform_impl;

pub use display::*;
pub use error::*;
pub use native_window::*;
pub use platform_event::*;
pub use platform_hook::*;
pub use platform_hook_installer::*;
#[cfg(target_os = "macos")]
pub use platform_impl::{
  print_all_app_window_titles, DisplayDeviceExtMacOs, DisplayExtMacOs, NativeWindowExtMacOs,
};
#[cfg(target_os = "windows")]
pub use platform_impl::{DisplayDeviceExtWindows, DisplayExtWindows};
