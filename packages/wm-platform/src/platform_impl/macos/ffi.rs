use std::{ffi::c_void, ptr::NonNull};

use objc2_application_services::{AXError, AXUIElement};
use objc2_core_foundation::CFUUID;
use objc2_core_graphics::{CGDirectDisplayID, CGError, CGWindowID};

use crate::platform_impl::ProcessId;

#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct ProcessSerialNumber {
  high: u32,
  low: u32,
}

#[derive(Default)]
#[repr(C, packed(2))]
pub(crate) struct ProcessInfo {
  pub(crate) info_length: u32,
  name: *const u8,
  psn: ProcessSerialNumber,
  pub(crate) r#type: u32,
  signature: u32,
  mode: u32,
  location: *const u8,
  size: u32,
  free_mem: u32,
  launcher: ProcessSerialNumber,
  launch_date: u32,
  active_time: u32,
  app_ref: *const u8,
}

pub const CPS_USER_GENERATED: u32 = 0x200;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
  // Deprecated in macOS 10.9 in late 2014, but still works fine.
  pub(crate) fn GetProcessForPID(
    pid: ProcessId,
    psn: *mut ProcessSerialNumber,
  ) -> u32;

  // Deprecated in macOS 10.9 in late 2014, but still works fine.
  pub(crate) fn GetProcessInformation(
    psn: *const ProcessSerialNumber,
    process_info: *mut ProcessInfo,
  ) -> u32;

  // `CGDisplayCreateUUIDFromDisplayID` comes from the `ColorSync`
  // framework, which is a subframework of `ApplicationServices`.
  pub(crate) fn CGDisplayCreateUUIDFromDisplayID(
    display: CGDirectDisplayID,
  ) -> Option<NonNull<CFUUID>>;
}

unsafe extern "C" {
  pub(crate) fn _AXUIElementGetWindow(
    elem: NonNull<AXUIElement>,
    window_id: *mut CGWindowID,
  ) -> AXError;
}

#[link(name = "SkyLight", kind = "framework")]
unsafe extern "C" {
  pub(crate) fn _SLPSSetFrontProcessWithOptions(
    psn: &ProcessSerialNumber,
    window_id: i32,
    mode: u32,
  ) -> CGError;

  pub(crate) fn SLPSPostEventRecordTo(
    psn: &ProcessSerialNumber,
    event: *const c_void,
  ) -> CGError;
}
