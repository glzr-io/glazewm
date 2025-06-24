#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![feature(iterator_try_collect)]
#![feature(once_cell_try)]

mod native_window;
mod platform_impl;

pub use native_window::*;
pub use platform_impl::*;
