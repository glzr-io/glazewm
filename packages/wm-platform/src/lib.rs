#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![feature(iterator_try_collect)]

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

#[cfg(feature = "test_utils")]
pub mod test_utils;

pub use dispatcher::*;
pub use display::*;
pub use display_listener::*;
pub use error::*;
pub use event_loop::*;
pub use keybinding_listener::*;
pub use models::*;
pub use mouse_listener::*;
pub use native_window::*;
#[cfg(target_os = "windows")]
mod native_surrogate;
#[cfg(target_os = "windows")]
pub use native_surrogate::NativeSurrogate;
#[cfg(target_os = "windows")]
mod resize_session;
#[cfg(target_os = "windows")]
pub use resize_session::ResizeSession;
#[cfg(target_os = "windows")]
mod workspace_surrogate;
#[cfg(target_os = "windows")]
pub use workspace_surrogate::WorkspaceSurrogate;


pub use platform_event::*;
pub use single_instance::*;
pub use thread_bound::*;
pub use window_listener::*;
/// Waits for the next DWM composition frame to complete.
///
/// Used to synchronize animation ticks to vsync so surrogate updates reach
/// the compositor on every rendered frame without timer-resolution jitter.
/// On non-Windows platforms this is a no-op.
pub fn dwm_flush() {
  #[cfg(target_os = "windows")]
  unsafe {
    // SAFETY: No preconditions; `DwmFlush` is safe to call from any thread
    // and blocks until the next DWM composition frame is ready.
    let _ = windows::Win32::Graphics::Dwm::DwmFlush();
  }
}

// TODO: Avoid exposing `windows` crate types in the public API.
#[cfg(target_os = "windows")]
pub use windows::Win32::UI::WindowsAndMessaging::{
  SET_WINDOW_POS_FLAGS, SWP_ASYNCWINDOWPOS, SWP_FRAMECHANGED,
  SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOSENDCHANGING, SWP_NOZORDER,
  WINDOW_EX_STYLE, WINDOW_STYLE, WS_CAPTION, WS_CHILD, WS_EX_NOACTIVATE,
  WS_EX_TOOLWINDOW, WS_MAXIMIZEBOX,
};
