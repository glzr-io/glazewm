#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![feature(iterator_try_collect)]

mod dispatcher;
mod display;
mod display_listener;
mod error;
mod event_loop;
mod event_loop_installer;
mod keybinding_listener;
mod models;
mod mouse_listener;
mod native_window;
mod platform_event;
mod platform_impl;
mod single_instance;
mod thread_bound;
mod window_listener;

pub use dispatcher::*;
pub use display::*;
pub use display_listener::*;
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
pub use platform_impl::{
  DisplayDeviceExtWindows, DisplayExtWindows, NativeWindowWindowsExt,
  WndProcCallback,
};
pub use single_instance::*;
pub use thread_bound::*;
pub use window_listener::*;

/// Window corner style.
///
/// # Platform-specific
///
/// Only has visible effect on Windows 11 and later.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CornerStyle {
  Default,
  Square,
  Rounded,
  SmallRounded,
}

#[cfg(target_os = "windows")]
pub use windows::Win32::UI::WindowsAndMessaging::{
  SET_WINDOW_POS_FLAGS, SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED,
  SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOSENDCHANGING, WINDOW_EX_STYLE,
  WINDOW_STYLE, WS_CAPTION, WS_CHILD, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
  WS_MAXIMIZEBOX,
};
