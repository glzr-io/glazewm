use std::ptr::NonNull;

use objc2_application_services::{AXError, AXUIElement};
use objc2_core_graphics::CGWindowID;

use crate::platform_impl::{AXUIElementRef, ProcessId};

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
  pub fn AXUIElementCreateApplication(pid: ProcessId) -> AXUIElementRef;
}

unsafe extern "C" {
  pub fn _AXUIElementGetWindow(
    elem: NonNull<AXUIElement>,
    wid: *mut CGWindowID,
  ) -> AXError;
}
