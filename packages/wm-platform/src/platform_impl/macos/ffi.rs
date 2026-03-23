use std::{ffi::c_void, ptr::NonNull};

use objc2_application_services::{AXError, AXUIElement};
use objc2_core_foundation::{CGAffineTransform, CFUUID};
use objc2_core_graphics::{CGDirectDisplayID, CGError, CGWindowID};

use crate::platform_impl::ProcessId;

/// Carbon process serial number (PSN), used to uniquely identify a
/// process.
#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct ProcessSerialNumber {
  high: u32,
  low: u32,
}

/// Carbon process information, populated by `GetProcessInformation`.
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

  /// Returns the connection ID for the current process's SkyLight session.
  pub(crate) fn SLSMainConnectionID() -> i32;

  /// Creates a new SkyLight display transaction.
  pub(crate) fn SLSTransactionCreate(cid: i32) -> *mut c_void;

  /// Applies an affine transform to a window within a transaction.
  pub(crate) fn SLSTransactionSetWindowTransform(
    transaction: *mut c_void,
    wid: u32,
    unknown1: i32,
    unknown2: i32,
    transform: CGAffineTransform,
  ) -> CGError;

  /// Sets the alpha (opacity) of a window within a transaction.
  pub(crate) fn SLSTransactionSetWindowAlpha(
    transaction: *mut c_void,
    wid: u32,
    alpha: f64,
  ) -> CGError;

  /// Commits a transaction, applying all queued operations.
  ///
  /// `synchronous` controls whether to wait for completion (1) or
  /// return immediately (0).
  pub(crate) fn SLSTransactionCommit(
    transaction: *mut c_void,
    synchronous: i32,
  ) -> CGError;

  pub fn SLSDisableUpdate(cid: i32) -> i32;
  pub fn SLSReenableUpdate(cid: i32) -> i32;
}
