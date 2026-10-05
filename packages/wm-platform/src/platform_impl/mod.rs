#[path = "windows/mod.rs"]
mod platform;

pub(crate) use platform::*;

#[cfg(not(target_os = "windows"))]
compile_error!("GlazeWM only supports Windows.");
