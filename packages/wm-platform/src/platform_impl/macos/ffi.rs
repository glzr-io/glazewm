use std::{ffi::c_void, ptr::NonNull};

use objc2_application_services::{AXError, AXUIElement};
use objc2_core_foundation::{CGRect, CFUUID};
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

pub(crate) type SLSConnection = i32;
pub(crate) type SLSWindow = u32;
#[allow(dead_code)]
pub(crate) type CGContextRef = *mut c_void;

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
  /// Returns the main `SkyLight` connection ID for the current process.
  pub(crate) fn SLSMainConnectionID() -> SLSConnection;

  /// Creates a new `SkyLight` connection.
  #[allow(dead_code)]
  pub(crate) fn SLSNewConnection(
    unknown: i32,
    conn: *mut SLSConnection,
  ) -> i32;

  /// Creates a new `SkyLight` window for the given region.
  pub(crate) fn SLSNewWindow(
    conn: SLSConnection,
    r#type: i32,
    x: f32,
    y: f32,
    region: *const c_void,
    wid: *mut SLSWindow,
  ) -> i32;

  /// Sets tags on a `SkyLight` window (e.g. floating, click-through).
  pub(crate) fn SLSSetWindowTags(
    conn: SLSConnection,
    wid: SLSWindow,
    tags: *const u64,
    tag_size: i32,
  ) -> i32;

  /// Clears tags on a `SkyLight` window.
  pub(crate) fn SLSClearWindowTags(
    conn: SLSConnection,
    wid: SLSWindow,
    tags: *const u64,
    tag_size: i32,
  ) -> i32;

  /// Releases a `SkyLight` window.
  pub(crate) fn SLSReleaseWindow(
    conn: SLSConnection,
    wid: SLSWindow,
  ) -> i32;

  /// Sets a `SkyLight` window's shape using a region.
  pub(crate) fn SLSSetWindowShape(
    conn: SLSConnection,
    wid: SLSWindow,
    x: f32,
    y: f32,
    shape: *const c_void,
  ) -> i32;

  /// Orders a `SkyLight` window relative to another window.
  pub(crate) fn SLSOrderWindow(
    conn: SLSConnection,
    wid: SLSWindow,
    mode: i32,
    ref_wid: SLSWindow,
  ) -> i32;

  /// Sets whether a `SkyLight` window is opaque.
  pub(crate) fn SLSSetWindowOpacity(
    conn: SLSConnection,
    wid: SLSWindow,
    opaque: bool,
  ) -> i32;

  /// Sets a `SkyLight` window's rendering resolution scale.
  pub(crate) fn SLSSetWindowResolution(
    conn: SLSConnection,
    wid: SLSWindow,
    res: f64,
  ) -> i32;

  /// Configures shadow properties for a `SkyLight` window.
  pub(crate) fn SLSWindowSetShadowProperties(
    wid: SLSWindow,
    props: *const c_void,
  ) -> i32;

  /// Creates a drawing context for a `SkyLight` window.
  pub(crate) fn SLWindowContextCreate(
    conn: SLSConnection,
    wid: SLSWindow,
    options: *const c_void,
  ) -> CGContextRef;

  /// Creates a Core Graphics region from a rectangle.
  pub(crate) fn CGSNewRegionWithRect(
    rect: *const CGRect,
    region: *mut *const c_void,
  ) -> i32;

  /// Releases a Core Graphics region.
  pub(crate) fn CGSReleaseRegion(region: *const c_void) -> i32;

  /// Brings a process to the front with `SkyLight` options.
  pub(crate) fn _SLPSSetFrontProcessWithOptions(
    psn: &ProcessSerialNumber,
    window_id: i32,
    mode: u32,
  ) -> CGError;

  /// Posts an input event record to a process.
  pub(crate) fn SLPSPostEventRecordTo(
    psn: &ProcessSerialNumber,
    event: *const c_void,
  ) -> CGError;
}
