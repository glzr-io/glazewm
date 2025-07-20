use std::{ffi::c_void, ptr::NonNull};

use anyhow::{bail, Context};
// use core_graphics::event::{
//   CGEventTap, CGEventTapLocation, CGEventTapOptions,
// CGEventTapPlacement,   CGEventType, CallbackResult,
// };
use objc2::{msg_send, rc::Retained, ClassType, MainThreadMarker};
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use objc2_core_foundation::{
  kCFRunLoopCommonModes, CFMachPort, CFRunLoop,
};
use objc2_core_graphics::{
  CGEvent, CGEventMask, CGEventTapLocation, CGEventTapOptions,
  CGEventTapPlacement, CGEventTapProxy, CGEventType,
};
use objc2_foundation::{NSString, NSThread};
use tokio::sync::mpsc;

use crate::{
  platform_impl::{EventLoop, EventLoopDispatcher},
  PlatformHookInstaller,
};

pub struct PlatformHook {
  dispatcher: Option<EventLoopDispatcher>,
}

impl PlatformHook {
  /// Creates a new `PlatformHook` instance and returns an installer for
  /// setting up the platform-specific event loop integration.
  ///
  /// The `PlatformHook` can be used immediately to create listeners and
  /// query system state, but events will only be received after the
  /// `PlatformHookInstaller` has been used to integrate with an event
  /// loop.
  ///
  /// # Example usage
  ///
  /// ```rust
  /// // Create hook and installer.
  /// let (hook, installer) = PlatformHook::new();
  ///
  /// // Set up listeners.
  /// let mouse_listener = hook.create_mouse_listener()?;
  ///
  /// // Install on platform-specific event loop.
  /// installer.run_dedicated_loop(); // Blocks until shutdown.
  /// ```
  #[must_use]
  pub fn new() -> (Self, PlatformHookInstaller) {
    let (installed_tx, installed_rx) = mpsc::unbounded_channel();
    let installer = PlatformHookInstaller::new(installed_tx);

    (Self { dispatcher: None }, installer)
  }

  // /// Creates a new [`MouseListener`] instance.
  // ///
  // /// This method will wait for the platform hook to be installed.
  // pub async fn create_mouse_listener(
  //   &mut self,
  // ) -> anyhow::Result<MouseListener> {
  //   let dispatcher = self.resolve_dispatcher().await?;
  //   MouseListener::new(dispatcher)
  // }
}

impl Drop for PlatformHook {
  fn drop(&mut self) {
    // TODO: Stop the event loop if installed via
    // `PlatformHookInstaller::run_dedicated_loop`.
  }
}

pub fn list_apps() -> impl Iterator<Item = (String, String)> {
  unsafe { NSWorkspace::sharedWorkspace().runningApplications() }
    .into_iter()
    .flat_map(move |app| {
      let bundle_id = unsafe { app.bundleIdentifier() }?.to_string();
      let localized_name = unsafe { app.localizedName() }?.to_string();
      Some((bundle_id, localized_name))
    })
}

fn create_event_tap() -> anyhow::Result<()> {
  let mask: CGEventMask = 1u64 << CGEventType::LeftMouseDown.0 as u64
    | 1u64 << CGEventType::LeftMouseUp.0 as u64
    | 1u64 << CGEventType::RightMouseDown.0 as u64
    | 1u64 << CGEventType::RightMouseUp.0 as u64
    | 1u64 << CGEventType::MouseMoved.0 as u64
    | 1u64 << CGEventType::LeftMouseDragged.0 as u64
    | 1u64 << CGEventType::RightMouseDragged.0 as u64;

  let tap = unsafe {
    CGEvent::tap_create(
      CGEventTapLocation::SessionEventTap,
      CGEventTapPlacement::HeadInsertEventTap,
      CGEventTapOptions::ListenOnly,
      mask,
      Some(raw_callback),
      std::ptr::null_mut(),
    )
  }
  .context("Failed to create event tap")?;

  let loop_ = CFMachPort::new_run_loop_source(None, Some(&tap), 0)
    .ok_or(anyhow::anyhow!("Failed to create loop source"))?;

  let current_loop = CFRunLoop::current().unwrap();
  current_loop.add_source(Some(&loop_), unsafe { kCFRunLoopCommonModes });

  unsafe { CGEvent::tap_enable(&tap, true) };

  Ok(())
}

unsafe extern "C-unwind" fn raw_callback(
  _proxy: CGEventTapProxy,
  _type: CGEventType,
  cg_event: NonNull<CGEvent>,
  _user_info: *mut c_void,
) -> *mut CGEvent {
  println!("Event: {:?}", _type);
  cg_event.as_ptr()
}

fn is_main_thread() -> bool {
  unsafe {
    let is_main: bool = msg_send![NSThread::class(), isMainThread];
    is_main
  }
}
