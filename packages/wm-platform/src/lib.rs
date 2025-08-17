#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![feature(iterator_try_collect)]
#![feature(once_cell_try)]

mod display;
mod error;
mod native_monitor;
mod native_window;
mod platform_event;
pub mod platform_ext;
mod platform_hook;
mod platform_hook_installer;
mod platform_impl;

pub use display::*;
pub use error::*;
pub use native_monitor::*;
pub use native_window::*;
pub use platform_event::*;
pub use platform_hook::*;
pub use platform_hook_installer::*;
