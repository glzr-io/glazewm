#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![feature(iterator_try_collect)]
#![feature(once_cell_try)]

mod com;
mod event_listener;
mod event_window;
mod keyboard_hook;
mod mouse_hook; 
mod native_monitor;
mod native_window;
mod platform;
mod single_instance;
mod window_event_hook;

pub use com::*;
pub use event_listener::*;
pub use event_window::*;
pub use keyboard_hook::*;
pub use mouse_hook::*; 
pub use native_monitor::*;
pub use native_window::*;
pub use platform::*;
pub use single_instance::*;
pub use window_event_hook::*;