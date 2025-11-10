use std::{fmt, os::raw::c_void, ptr::NonNull};

use objc2_app_kit::NSEvent;
use objc2_core_foundation::{
  kCFRunLoopCommonModes, CFMachPort, CFRetained, CFRunLoop,
};
use objc2_core_graphics::{
  CGEvent, CGEventField, CGEventMask, CGEventTapLocation,
  CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
};

use crate::{
  platform_event::MouseEventNotification, Dispatcher, Error, MouseButton,
  MouseEventType, Point, ThreadBound, WindowId,
};

/// macOS-specific implementation of [`MouseEventNotification`].
#[derive(Clone, Debug)]
pub struct MouseEventNotificationInner {
  pub event_type: CGEventType,
  pub event: NonNull<CGEvent>,
}

impl MouseEventNotificationInner {
  /// NOTE: Unfortunately quite unreliable. Returns 0 in most cases with
  /// the real window ID interspersed every so often. Often a 100-200ms
  /// delay before the real window ID is returned.
  #[must_use]
  pub fn below_window_id(&self) -> Option<WindowId> {
    let window_id = unsafe {
      CGEvent::integer_value_field(
        Some(self.event.as_ref()),
        CGEventField::MouseEventWindowUnderMousePointer,
      )
    };

    if window_id == 0 {
      None
    } else {
      #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
      Some(WindowId(window_id as u32))
    }
  }

  pub fn event_type(&self) -> MouseEventType {
    match self.event_type {
      CGEventType::MouseMoved => MouseEventType::Move,
      CGEventType::LeftMouseDragged => MouseEventType::Move,
      CGEventType::RightMouseDragged => MouseEventType::Move,
      CGEventType::LeftMouseDown => MouseEventType::LeftClickDown,
      CGEventType::LeftMouseUp => MouseEventType::LeftClickUp,
      CGEventType::RightMouseDown => MouseEventType::RightClickDown,
      CGEventType::RightMouseUp => MouseEventType::RightClickUp,
      _ => unreachable!(),
    }
  }

  pub fn position(&self) -> Point {
    let cg_point = unsafe { CGEvent::location(Some(self.event.as_ref())) };

    #[allow(clippy::cast_possible_truncation)]
    Point {
      x: cg_point.x as i32,
      y: cg_point.y as i32,
    }
  }

  pub fn pressed_buttons(&self) -> Vec<MouseButton> {
    let mut pressed_buttons = Vec::new();

    let pressed_mask = unsafe { NSEvent::pressedMouseButtons() };
    if pressed_mask & 1 != 0 {
      pressed_buttons.push(MouseButton::Left);
    }
    if pressed_mask & 2 != 0 {
      pressed_buttons.push(MouseButton::Right);
    }

    pressed_buttons
  }
}

unsafe impl Send for MouseEventNotificationInner {}

/// A callback invoked for every mouse notification received by the
/// system `CGEventTap`.
type HookCallback = dyn Fn(MouseEventNotification) + Send + Sync + 'static;

/// Wrapper for the opaque callback pointer passed to C callbacks.
///
/// Ensures the pointer can be moved across threads (`Send + Sync`) and
/// is properly reclaimed on drop.
struct CallbackUserData {
  // ptr: Box<*mut c_void>,
  // ptr: Box<HookCallback>,
  ptr: *mut c_void,
}

impl CallbackUserData {
  #[inline]
  fn new(callback: *mut c_void) -> Self {
    // SAFETY: Caller must ensure `ptr` is non-null and points to memory
    // allocated by `Box::into_raw(Box<HookCallback>)`.
    Self {
      ptr: callback,
      // ptr: Box::new(
      //   NonNull::new(ptr).expect("callback pointer must be non-null"),
      // ),
      // ptr: Box::new(
      //   ptr, /* NonNull::new(ptr).expect("callback pointer must be
      //        * non-null"), */
      // ),
    }
    // fn new(ptr: *mut c_void) -> Self {
    //   // SAFETY: Caller must ensure `ptr` is non-null and points to
    // memory   // allocated by `Box::into_raw(Box<HookCallback>)`.
    //   Self {
    //     // ptr: Box::new(
    //     //   NonNull::new(ptr).expect("callback pointer must be
    // non-null"),     // ),
    //     ptr: Box::new(
    //       ptr, /* NonNull::new(ptr).expect("callback pointer must be
    //            * non-null"), */
    //     ),
    //   }
  }

  fn as_ptr(&self) -> *mut c_void {
    // Box::into_raw(&self.ptr).cast::<c_void>()
    self.ptr
  }

  // #[inline]
  // fn as_ptr(&self) -> *mut c_void {
  //   (&self.ptr).as
  // }
}

// SAFETY: The pointer refers to a `Box<HookCallback>` which is `Send +
// Sync + 'static`. We never mutate the Box here; the pointer is only
// passed as opaque user data to the C callback, and reclaimed exactly once
// in `Drop`.
unsafe impl Send for CallbackUserData {}
// unsafe impl Sync for CallbackUserData {}

impl Drop for CallbackUserData {
  fn drop(&mut self) {
    // SAFETY: Pointer was created with `Box::into_raw` in `MouseHook::new`
    // and we are the unique owner in this wrapper. This converts it
    // back and drops.
    unsafe {
      // let _ = Box::from_raw(*self.ptr.cast::<Box<HookCallback>>());
    }
    println!("CallbackUserData dropped");
  }
}

impl fmt::Debug for CallbackUserData {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("CallbackUserData").finish_non_exhaustive()
  }
}

/// macOS-specific mouse hook that listens for configured mouse events and
/// executes a provided callback for each notification.
// #[derive(Debug)]
pub struct MouseHook {
  /// Mach port for the created `CGEventTap`.
  tap_port: Option<ThreadBound<CFRetained<CFMachPort>>>,
  // /// Opaque pointer to the boxed callback state passed to the C
  // callback. user_data: Option<CallbackUserData>,
  // _test: Box<HookCallback>,
  // callback_ptr: *mut c_void,
  // _callback_ptr: CallbackPtr,
  // callback: Box<HookCallback>,
  // callback_ptr: *mut c_void,
}

#[derive(Clone)]
struct CallbackPtr(*mut c_void);
impl CallbackPtr {
  fn as_ptr(&self) -> *mut c_void {
    self.0
  }
}

unsafe impl Send for CallbackPtr {}

// struct CallbackPtr(*mut c_void);
// unsafe impl Send for CallbackPtr {}

impl MouseHook {
  /// Creates a new mouse hook with the specified enabled mouse event
  /// kinds and callback.
  ///
  /// The callback is executed for every received mouse notification.
  pub fn new<F>(
    dispatcher: &Dispatcher,
    enabled_events: &[crate::mouse_listener::MouseEventType],
    callback: F,
  ) -> crate::Result<Self>
  where
    F: Fn(MouseEventNotification) + Send + Sync + 'static,
  {
    // let callback: Box<HookCallback> = Box::new(callback);
    // Box the callback and pass the raw pointer as user data to the C
    // callback.
    // let callback_ptr = Box::into_raw(callback).cast::<c_void>();
    // let wrapped_callback_ptr = CallbackPtr(callback_ptr);

    // Wrap pointer in a Send + Sync owner that will free it on drop.
    // let user_data = CallbackUserData::new(callback_ptr);
    // let test = Box::new(callback);
    // let user_data = CallbackUserData::new(&raw const test as *mut
    // c_void);

    // let tap_port = match dispatcher.dispatch_sync(move || {
    //   Self::create_event_tap::<F>(
    //     dispatcher,
    //     enabled_events,
    //     user_data.as_ptr(),
    //   )
    // }) {
    //   Ok(Ok(port)) => port,
    //   Ok(Err(err)) | Err(err) => {
    //     // `user_data` drops here and frees the boxed callback.
    //     return Err(err);
    //   }
    // };

    // Box the callback and convert to raw pointer - this keeps it alive
    // let callback_ptr = Box::into_raw(Box::new(callback)) as *mut c_void;
    // let callback: Box<HookCallback> = Box::new(callback);
    // let callback: Box<HookCallback> = Box::new(callback);
    // let callback: Box<HookCallback> = Box::new(Box::new(callback));
    // let callback_ptr = Box::into_raw(callback).cast::<c_void>();

    // // let callback_ptr_clone = callback_ptr.clone();
    // let xx = CallbackPtr(callback_ptr);
    let tap_port = match dispatcher.dispatch_sync(|| {
      Self::create_event_tap::<F>(
        dispatcher,
        enabled_events,
        // Box::new(callback),
        Box::new(callback),
      )
    }) {
      Ok(Ok(port)) => port,
      Ok(Err(err)) | Err(err) => {
        // Clean up the callback on error
        // unsafe {
        //   let _ = Box::from_raw(callback_ptr as *mut F);
        // }
        return Err(err);
      }
    };

    Ok(Self {
      tap_port: Some(tap_port),
      // user_data: Some(user_data),
      // _test: test,
      // _callback_ptr: callback_ptr,
      // callback,
      // callback_ptr,
    })
  }

  /// Creates and registers a `CGEventTap` for mouse events on the run
  /// loop.
  fn create_event_tap<F>(
    dispatcher: &Dispatcher,
    enabled_events: &[crate::mouse_listener::MouseEventType],
    callback: Box<F>,
  ) -> crate::Result<ThreadBound<CFRetained<CFMachPort>>>
  where
    F: Fn(MouseEventNotification) + Send + Sync + 'static,
  {
    let mask = Self::event_mask_from_enabled(enabled_events);
    let user_data_ptr = Box::into_raw(callback).cast::<c_void>();

    let tap_port = unsafe {
      CGEvent::tap_create(
        CGEventTapLocation::AnnotatedSessionEventTap,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        mask,
        Some(Self::mouse_event_callback::<F>),
        user_data_ptr,
        // Box::into_raw(user_data_ptr.ptr).cast::<c_void>(),
      )
      .ok_or_else(|| {
        Error::Platform(
          "Failed to create CGEventTap. Accessibility permissions may be required.".to_string(),
        )
      })
    }?;

    let loop_source =
      CFMachPort::new_run_loop_source(None, Some(&tap_port), 0)
        .ok_or_else(|| {
          Error::Platform("Failed to create loop source".to_string())
        })?;

    let current_loop = CFRunLoop::current().ok_or_else(|| {
      Error::Platform("Failed to get current run loop".to_string())
    })?;

    current_loop
      .add_source(Some(&loop_source), unsafe { kCFRunLoopCommonModes });

    unsafe { CGEvent::tap_enable(&tap_port, true) };

    Ok(ThreadBound::new(tap_port, dispatcher.clone()))
  }

  /// Enables or disables the underlying event tap.
  pub fn enable(&mut self, enabled: bool) {
    if let Some(tap_port) = &self.tap_port {
      let _ =
        tap_port.with(|tap| unsafe { CGEvent::tap_enable(tap, enabled) });
    }
  }

  /// Terminates the hook by invalidating the event tap and cleaning up
  /// resources.
  #[allow(clippy::unnecessary_wraps)]
  pub fn terminate(&mut self) -> crate::Result<()> {
    if let Some(tap) = self.tap_port.take() {
      // Invalidate the event tap to stop it from receiving events. This
      // also invalidates the run loop source.
      // See: https://developer.apple.com/documentation/corefoundation/cfmachportinvalidate(_:)
      let _ = tap.with(|tap| CFMachPort::invalidate(tap));
    }

    Ok(())
  }

  /// Converts enabled high-level mouse events into a `CGEvent` mask.
  fn event_mask_from_enabled(
    enabled_events: &[crate::mouse_listener::MouseEventType],
  ) -> CGEventMask {
    let mut mask = 0u64;

    for event in enabled_events {
      match event {
        crate::mouse_listener::MouseEventType::Move => {
          mask |= 1u64 << u64::from(CGEventType::MouseMoved.0);
          mask |= 1u64 << u64::from(CGEventType::LeftMouseDragged.0);
          mask |= 1u64 << u64::from(CGEventType::RightMouseDragged.0);
        }
        crate::mouse_listener::MouseEventType::LeftClickDown => {
          mask |= 1u64 << u64::from(CGEventType::LeftMouseDown.0);
        }
        crate::mouse_listener::MouseEventType::RightClickDown => {
          mask |= 1u64 << u64::from(CGEventType::RightMouseDown.0);
        }
        crate::mouse_listener::MouseEventType::LeftClickUp => {
          mask |= 1u64 << u64::from(CGEventType::LeftMouseUp.0);
        }
        crate::mouse_listener::MouseEventType::RightClickUp => {
          mask |= 1u64 << u64::from(CGEventType::RightMouseUp.0);
        }
      }
    }

    mask
  }

  /// Callback for mouse `CGEventTap`.
  extern "C-unwind" fn mouse_event_callback<F>(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    mut event: NonNull<CGEvent>,
    user_info: *mut c_void,
  ) -> *mut CGEvent
  where
    F: Fn(MouseEventNotification) + Send + Sync + 'static,
  {
    if user_info.is_null() {
      tracing::error!("Null pointer passed to mouse event callback.");
      return unsafe { event.as_mut() };
    }

    // Build notification and invoke the user callback for all enabled
    // events.
    let notification =
      MouseEventNotification(MouseEventNotificationInner {
        event_type,
        event,
      });

    // SAFETY: `user_info` points to a boxed `HookCallback`.
    // let callback = unsafe { &*(user_info as *mut c_void) };
    // let callback = unsafe { &*(user_info as *const Box<HookCallback>) };

    println!("user_info1");
    let callback = unsafe { &*(user_info as *const F) };
    println!("user_info2");
    (callback)(notification);
    println!("user_info3");

    unsafe { event.as_mut() }
  }
}

impl Drop for MouseHook {
  fn drop(&mut self) {
    let _ = self.terminate();
  }
}
