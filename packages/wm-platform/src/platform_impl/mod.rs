#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod platform;
#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod platform;

pub use platform::*;

#[cfg(all(not(target_os = "windows"), not(target_os = "macos"),))]
compile_error!("The platform you're compiling for is not supported.");
