mod native_window;
mod platform_hook;

pub(crate) mod event_loop;
pub(crate) mod grabs;
pub(crate) mod handlers;
pub(crate) mod input;
pub(crate) mod key;
pub(crate) mod state;
pub(crate) mod windows;
pub(crate) mod winit;

mod hooks;

mod native_monitor;

pub use hooks::*;
pub use native_monitor::*;
pub use native_window::*;
pub use platform_hook::*;
use smithay::reexports::wayland_server::DisplayHandle;
pub use wm_common::WindowHandle;

pub struct CalloopData {
  pub state: state::Glaze,
  pub display_handle: DisplayHandle,
}
