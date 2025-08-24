use std::{ffi::c_void, ptr::NonNull};

use anyhow::Context;
use objc2::{msg_send, ClassType};
use objc2_app_kit::NSWorkspace;
use objc2_core_foundation::{
  kCFRunLoopCommonModes, CFMachPort, CFRunLoop,
};
use objc2_core_graphics::{
  CGEvent, CGEventMask, CGEventTapLocation, CGEventTapOptions,
  CGEventTapPlacement, CGEventTapProxy, CGEventType,
};
use objc2_foundation::NSThread;
use tokio::sync::oneshot;
use wm_common::Point;

use crate::{
  platform_impl::{self, EventLoopDispatcher, WindowListener},
  Display, DisplayDevice, NativeWindow, PlatformHookInstaller,
};

pub struct PlatformHook {
  dispatcher_rx: Option<oneshot::Receiver<EventLoopDispatcher>>,
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
    let (dispatcher_tx, dispatcher_rx) = oneshot::channel();
    let installer = PlatformHookInstaller::new(dispatcher_tx);

    (
      Self {
        dispatcher_rx: Some(dispatcher_rx),
        dispatcher: None,
      },
      installer,
    )
  }

  /// Resolves the event loop dispatcher, waiting for it to be available if
  /// necessary.
  async fn resolve_dispatcher(
    &mut self,
  ) -> anyhow::Result<&EventLoopDispatcher> {
    if let Some(ref dispatcher) = self.dispatcher {
      return Ok(dispatcher);
    }

    let dispatcher_rx = self
      .dispatcher_rx
      .take()
      .context("Dispatcher receiver has already been consumed.")?;

    let dispatcher = dispatcher_rx
      .await
      .context("Failed to receive dispatcher from installer.")?;

    // Insert and get reference in one go.
    Ok(self.dispatcher.insert(dispatcher))
  }

  /// Creates a new [`MouseListener`] instance.
  ///
  /// This method will wait for the platform hook to be installed.
  pub async fn create_mouse_listener(&mut self) -> anyhow::Result<()> {
    let _ = self.resolve_dispatcher().await?;
    todo!()
  }

  /// Creates a new [`WindowListener`] instance.
  ///
  /// This method will wait for the platform hook to be installed.
  pub async fn create_window_listener(
    &mut self,
  ) -> anyhow::Result<WindowListener> {
    let dispatcher = self.resolve_dispatcher().await?;
    WindowListener::new(dispatcher)
  }

  /// Creates a new [`KeybindingListener`] instance.
  ///
  /// This method will wait for the platform hook to be installed.
  pub async fn create_keybinding_listener(
    &mut self,
  ) -> anyhow::Result<()> {
    let _ = self.resolve_dispatcher().await?;
    todo!()
  }

  /// Gets all active displays.
  ///
  /// Returns all displays that are currently active and available for use.
  pub async fn displays(&mut self) -> crate::Result<Vec<Display>> {
    let dispatcher = self.resolve_dispatcher().await?;
    platform_impl::all_displays(dispatcher)
  }

  /// Gets all display devices.
  ///
  /// Returns all display devices including active, inactive, and
  /// disconnected ones.
  pub async fn all_display_devices(
    &mut self,
  ) -> crate::Result<Vec<DisplayDevice>> {
    let dispatcher = self.resolve_dispatcher().await?;
    platform_impl::all_display_devices(dispatcher)
  }

  /// Gets the display containing the specified point.
  ///
  /// If no display contains the point, returns the primary display.
  pub async fn display_from_point(
    &mut self,
    point: Point,
  ) -> crate::Result<Display> {
    let dispatcher = self.resolve_dispatcher().await?;
    platform_impl::display_from_point(point, dispatcher)
  }

  /// Gets the primary display.
  ///
  /// # Platform-specific
  ///
  /// - **macOS**: Returns the display containing the menu bar.
  pub async fn primary_display(&mut self) -> crate::Result<Display> {
    let dispatcher = self.resolve_dispatcher().await?;
    platform_impl::primary_display(dispatcher)
  }

  /// Gets all windows.
  ///
  /// Returns all windows that are currently on-screen and meet the
  /// filtering criteria, excluding system windows like Dock, menu bar,
  /// and desktop elements.
  ///
  /// # Platform-specific
  ///
  /// - **macOS**: Uses `CGWindowListCopyWindowInfo` to enumerate windows
  ///   and filters out system applications and UI elements.
  pub async fn all_windows(&mut self) -> crate::Result<Vec<NativeWindow>> {
    let dispatcher = self.resolve_dispatcher().await?;
    platform_impl::all_windows(dispatcher)
  }

  pub async fn test(&mut self) -> crate::Result<()> {
    let dispatcher = self.resolve_dispatcher().await?;
    platform_impl::print_all_app_window_titles(dispatcher)
  }
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
