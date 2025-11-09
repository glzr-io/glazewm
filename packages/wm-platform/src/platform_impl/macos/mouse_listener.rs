use std::{os::raw::c_void, ptr::NonNull};

use objc2_app_kit::NSEvent;
use objc2_core_foundation::{
  kCFRunLoopCommonModes, CFMachPort, CFRetained, CFRunLoop,
};
use objc2_core_graphics::{
  CGEvent, CGEventField, CGEventMask, CGEventTapLocation,
  CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
};
use tokio::sync::mpsc;

use crate::{
  platform_event::MouseEvent, Dispatcher, Error, MouseEventNotification,
  Point, ThreadBound, WindowId,
};

/// macOS-specific mouse event notification.
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
}

unsafe impl Send for MouseEventNotificationInner {}

/// macOS-specific listener for system-wide mouse events.
#[derive(Debug)]
pub struct MouseListener {
  /// Receiver for outgoing mouse events.
  pub event_rx: mpsc::UnboundedReceiver<MouseEvent>,

  /// Mach port for the created `CGEventTap`.
  tap_port: Option<ThreadBound<CFRetained<CFMachPort>>>,
}

impl MouseListener {
  /// Creates a new mouse listener and starts the event tap.
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let tap_port = dispatcher
      .dispatch_sync(|| Self::create_event_tap(dispatcher, event_tx))??;

    Ok(Self {
      event_rx,
      tap_port: Some(tap_port),
    })
  }

  /// Creates and registers a `CGEventTap` for mouse events on the run
  /// loop.
  fn create_event_tap(
    dispatcher: &Dispatcher,
    event_tx: mpsc::UnboundedSender<MouseEvent>,
  ) -> crate::Result<ThreadBound<CFRetained<CFMachPort>>> {
    // Listen for mouse movement and drag events. Down/up are included to
    // ensure accurate pressed state via `pressedMouseButtons`.
    let mask: CGEventMask = (1u64 << u64::from(CGEventType::MouseMoved.0))
      | (1u64 << u64::from(CGEventType::LeftMouseDragged.0))
      | (1u64 << u64::from(CGEventType::RightMouseDragged.0))
      | (1u64 << u64::from(CGEventType::LeftMouseDown.0))
      | (1u64 << u64::from(CGEventType::LeftMouseUp.0))
      | (1u64 << u64::from(CGEventType::RightMouseDown.0))
      | (1u64 << u64::from(CGEventType::RightMouseUp.0));

    // Box the sender and pass as user data to the callback.
    let event_tx_ptr = Box::into_raw(Box::new(event_tx)).cast::<c_void>();

    let tap_port = unsafe {
      CGEvent::tap_create(
        CGEventTapLocation::AnnotatedSessionEventTap,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        mask,
        Some(Self::mouse_event_callback),
        event_tx_ptr,
      )
      .ok_or_else(|| {
        // Clean up the sender if event tap creation fails.
        let _ = Box::from_raw(event_tx_ptr.cast::<mpsc::UnboundedSender<MouseEvent>>());
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

  /// Enables or disables the event tap.
  pub fn enable(&mut self, enabled: bool) {
    if let Some(tap_port) = &self.tap_port {
      let _ =
        tap_port.with(|tap| unsafe { CGEvent::tap_enable(tap, enabled) });
    }
  }

  /// Terminates the mouse listener by invalidating the event tap.
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

  /// Callback for mouse `CGEventTap`.
  extern "C-unwind" fn mouse_event_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    mut event: NonNull<CGEvent>,
    user_info: *mut c_void,
  ) -> *mut CGEvent {
    if user_info.is_null() {
      tracing::error!("Null pointer passed to mouse event callback.");
      return unsafe { event.as_mut() };
    }

    // Only emit for movement/drag events. Other mouse events fall through.
    let is_move_event = matches!(
      event_type,
      CGEventType::MouseMoved
        | CGEventType::LeftMouseDragged
        | CGEventType::RightMouseDragged
    );

    if is_move_event {
      let cg_point = unsafe { CGEvent::location(Some(event.as_ref())) };

      // Determine if left or right mouse button is down.
      let pressed_mask = unsafe { NSEvent::pressedMouseButtons() };
      let is_mouse_down =
        (pressed_mask & ((1usize << 0) | (1usize << 1))) != 0;

      // Convert to platform point (truncate toward zero).
      #[allow(clippy::cast_possible_truncation)]
      let point = Point {
        x: cg_point.x as i32,
        y: cg_point.y as i32,
      };

      let mouse_event = MouseEvent {
        point,
        is_mouse_down,
        notification: MouseEventNotification(
          MouseEventNotificationInner { event_type, event },
        ),
      };

      // SAFETY: `user_info` points to a boxed
      // `UnboundedSender<MouseEvent>`.
      let tx = unsafe {
        &*(user_info as *const mpsc::UnboundedSender<MouseEvent>)
      };

      let _ = tx.send(mouse_event);
    }

    unsafe { event.as_mut() }
  }
}

impl Drop for MouseListener {
  fn drop(&mut self) {
    let _ = self.terminate();
  }
}
