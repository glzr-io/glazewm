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
use tokio::{sync::mpsc, task};
use tracing::warn;
use wm_common::{
  Color, CornerStyle, Delta, HideMethod, LengthValue, Memo, OpacityValue,
  Rect, RectDelta, WindowState,
};

use crate::platform_impl::{EventLoop, EventLoopDispatcher};

pub struct PlatformHook {
  event_loop: Option<EventLoop>,
  dispatcher: Option<EventLoopDispatcher>,
}

impl PlatformHook {
  /// Creates a new `PlatformHook` instance with its own event loop running
  /// on a separate, dedicated thread.
  ///
  /// ## Platform-specific
  ///
  /// - Windows: Creates a new Win32 message loop.
  /// - MacOS: Creates a new *main* run loop (`CFRunLoopGetMain()`). Note
  ///   that MacOS only allows a single main run loop in a process, so if
  ///   you'd like to run your own main run loop, set up the hook using
  ///   [`PlatformHook::remote`] instead.
  #[must_use]
  pub fn dedicated() -> anyhow::Result<Self> {
    let (event_loop, dispatcher) = EventLoop::new()?;

    dispatcher.dispatch(move || {
      println!("Hello, world! from dispatcher");
      println!("Running apps: {:?}", list_apps().collect::<Vec<_>>());
      println!("Creating event tap");
      create_event_tap().unwrap();
      println!("Is main thread: {}", is_main_thread());
    })?;

    println!("Running apps outside!");
    println!("Creating event tap outside");
    std::thread::sleep(std::time::Duration::from_secs(3));
    println!("Is main thread: {}", is_main_thread());
    create_event_tap().unwrap();
    // println!("Running apps: {:?}",
    // running_apps(None).collect::<Vec<_>>());

    Ok(Self {
      event_loop: Some(event_loop),
      dispatcher: Some(dispatcher),
    })
  }

  // /// Creates a new `PlatformHook` instance to be installed on a target
  // /// thread. The `PlatformHookInstaller` returned is used to install
  // /// the hook on the target thread.
  // #[must_use]
  // pub fn remote() -> (Self, PlatformHookInstaller) {
  //   let (installed_tx, installed_rx) = mpsc::unbounded_channel();
  //   let installer = PlatformHookInstaller::new(installed_tx);

  //   (
  //     Self {
  //       event_loop: None,
  //       dispatcher: None,
  //     },
  //     installer,
  //   )
  // }

  // /// Creates a new [`MouseListener`] instance.
  // ///
  // /// This method will wait for the platform hook to be installed.
  // pub async fn create_mouse_listener(
  //   &mut self,
  // ) -> anyhow::Result<MouseListener> {
  //   let dispatcher = self.resolve_dispatcher().await?;
  //   MouseListener::new(dispatcher)
  // }

  // async fn resolve_dispatcher(
  //   &mut self,
  // ) -> anyhow::Result<EventLoopDispatcher> {
  //   // TODO: Wait for the dispatcher to be installed.
  //   todo!()
  // }
}

impl Drop for PlatformHook {
  fn drop(&mut self) {
    if let Some(event_loop) = self.event_loop.take() {
      // event_loop.stop();
    }
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
