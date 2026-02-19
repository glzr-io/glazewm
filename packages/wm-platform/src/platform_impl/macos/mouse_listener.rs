use std::{
  os::raw::c_void,
  ptr::NonNull,
  time::{Duration, Instant},
};

use objc2_core_foundation::{
  kCFRunLoopCommonModes, CFMachPort, CFRetained, CFRunLoop,
};
use objc2_core_graphics::{
  CGEvent, CGEventField, CGEventMask, CGEventTapLocation,
  CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
};
use tokio::sync::mpsc;

use crate::{
  mouse_listener::MouseEventKind,
  platform_event::{MouseButton, MouseEvent, PressedButtons},
  Dispatcher, Error, Point, ThreadBound, WindowId,
};

/// Data shared with the `CGEventTap` callback.
struct CallbackData {
  event_tx: mpsc::UnboundedSender<MouseEvent>,

  /// Pressed button state tracked from events.
  pressed_buttons: PressedButtons,

  /// Timestamp of the last emitted `Move` event for throttling.
  last_move_emission: Option<Instant>,
}

impl CallbackData {
  fn new(event_tx: mpsc::UnboundedSender<MouseEvent>) -> Self {
    Self {
      event_tx,
      pressed_buttons: PressedButtons::default(),
      last_move_emission: None,
    }
  }
}

/// macOS-specific implementation of [`MouseListener`].
#[derive(Debug)]
pub(crate) struct MouseListener {
  dispatcher: Dispatcher,
  event_tx: mpsc::UnboundedSender<MouseEvent>,

  /// Mach port for the created `CGEventTap`.
  tap_port: Option<ThreadBound<CFRetained<CFMachPort>>>,

  /// Pointer to [`CallbackData`], used by the `CGEventTap` callback.
  callback_data_ptr: usize,
}

impl MouseListener {
  /// macOS-specific implementation of [`MouseListener::new`].
  pub(crate) fn new(
    enabled_events: &[MouseEventKind],
    event_tx: mpsc::UnboundedSender<MouseEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let callback_data_ptr = {
      let data = Box::new(CallbackData::new(event_tx.clone()));
      Box::into_raw(data) as usize
    };

    let tap_port = dispatcher
      .dispatch_sync(|| {
        Self::create_event_tap(
          enabled_events,
          callback_data_ptr,
          dispatcher,
        )
      })
      .flatten()
      .inspect_err(|_| {
        // Clean up the callback data if event tap creation fails.
        let _ =
          unsafe { Box::from_raw(callback_data_ptr as *mut CallbackData) };
      })?;

    Ok(Self {
      tap_port: Some(tap_port),
      callback_data_ptr,
      dispatcher: dispatcher.clone(),
      event_tx,
    })
  }

  /// Creates and registers a `CGEventTap` for mouse events.
  fn create_event_tap(
    enabled_events: &[MouseEventKind],
    callback_data_ptr: usize,
    dispatcher: &Dispatcher,
  ) -> crate::Result<ThreadBound<CFRetained<CFMachPort>>> {
    let mask = Self::event_mask_from_enabled(enabled_events);

    let tap_port = unsafe {
      CGEvent::tap_create(
        CGEventTapLocation::AnnotatedSessionEventTap,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        mask,
        Some(Self::mouse_event_callback),
        callback_data_ptr as *mut c_void,
      )
      .ok_or_else(|| {
        Error::Platform(
          "Failed to create `CGEventTap`. Accessibility permissions may be required.".to_string(),
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

  /// macOS-specific implementation of [`MouseListener::enable`].
  pub(crate) fn enable(&mut self, enabled: bool) -> crate::Result<()> {
    if let Some(tap_port) = &self.tap_port {
      tap_port.with(|tap| unsafe { CGEvent::tap_enable(tap, enabled) })?;
    }

    Ok(())
  }

  /// macOS-specific implementation of
  /// [`MouseListener::set_enabled_events`].
  pub(crate) fn set_enabled_events(
    &mut self,
    enabled_events: &[MouseEventKind],
  ) -> crate::Result<()> {
    let _ = self.terminate();

    let callback_data_ptr = {
      let data = Box::new(CallbackData::new(self.event_tx.clone()));
      Box::into_raw(data) as usize
    };

    let tap_port = self
      .dispatcher
      .dispatch_sync(|| {
        Self::create_event_tap(
          enabled_events,
          callback_data_ptr,
          &self.dispatcher,
        )
      })
      .flatten()
      .inspect_err(|_| {
        // Clean up the callback data if event tap creation fails.
        let _ =
          unsafe { Box::from_raw(callback_data_ptr as *mut CallbackData) };
      })?;

    self.callback_data_ptr = callback_data_ptr;
    self.tap_port = Some(tap_port);

    Ok(())
  }

  /// macOS-specific implementation of [`MouseListener::terminate`].
  pub(crate) fn terminate(&mut self) -> crate::Result<()> {
    if let Some(tap) = self.tap_port.take() {
      // Invalidate the tap to stop it from receiving events. This also
      // invalidates the run loop source.
      // See: https://developer.apple.com/documentation/corefoundation/cfmachportinvalidate(_:)
      tap.with(|tap| CFMachPort::invalidate(tap))?;
    }

    // Clean up the callback data if it exists.
    if self.callback_data_ptr != 0 {
      let _ = unsafe {
        Box::from_raw(self.callback_data_ptr as *mut CallbackData)
      };
      self.callback_data_ptr = 0;
    }

    Ok(())
  }

  /// Gets the `CGEvent` mask for the enabled mouse events.
  fn event_mask_from_enabled(
    enabled_events: &[MouseEventKind],
  ) -> CGEventMask {
    let mut mask = 0u64;

    for event in enabled_events {
      match event {
        MouseEventKind::Move => {
          // NOTE: `MouseMoved` doesn't get triggered when clicking and
          // dragging. Therefore, we also listen for `LeftMouseDragged`
          // and `RightMouseDragged` events.
          mask |= 1u64 << u64::from(CGEventType::MouseMoved.0);
          mask |= 1u64 << u64::from(CGEventType::LeftMouseDragged.0);
          mask |= 1u64 << u64::from(CGEventType::RightMouseDragged.0);
        }
        MouseEventKind::LeftButtonDown => {
          mask |= 1u64 << u64::from(CGEventType::LeftMouseDown.0);
        }
        MouseEventKind::RightButtonDown => {
          mask |= 1u64 << u64::from(CGEventType::RightMouseDown.0);
        }
        MouseEventKind::LeftButtonUp => {
          mask |= 1u64 << u64::from(CGEventType::LeftMouseUp.0);
        }
        MouseEventKind::RightButtonUp => {
          mask |= 1u64 << u64::from(CGEventType::RightMouseUp.0);
        }
      }
    }

    mask
  }

  /// Callback for the `CGEventTap`.
  extern "C-unwind" fn mouse_event_callback(
    _: CGEventTapProxy,
    cg_event_type: CGEventType,
    mut cg_event: NonNull<CGEvent>,
    user_info: *mut c_void,
  ) -> *mut CGEvent {
    if user_info.is_null() {
      tracing::error!("Null pointer passed to mouse event callback.");
      return unsafe { cg_event.as_mut() };
    }

    let data = unsafe { &mut *user_info.cast::<CallbackData>() };

    // Map a `CGEventType` to a `MouseEventKind`.
    let event_kind = match cg_event_type {
      CGEventType::LeftMouseDown => MouseEventKind::LeftButtonDown,
      CGEventType::LeftMouseUp => MouseEventKind::LeftButtonUp,
      CGEventType::RightMouseDown => MouseEventKind::RightButtonDown,
      CGEventType::RightMouseUp => MouseEventKind::RightButtonUp,
      _ => MouseEventKind::Move,
    };

    // Extract the cursor position from the `CGEvent`.
    let cg_event_ref = unsafe { cg_event.as_ref() };
    let position = {
      let cg_point = unsafe { CGEvent::location(Some(cg_event_ref)) };

      #[allow(clippy::cast_possible_truncation)]
      Point {
        x: cg_point.x as i32,
        y: cg_point.y as i32,
      }
    };

    // NOTE: Unfortunately quite unreliable. Returns 0 in most cases with
    // the real window ID interspersed every so often. Often a 100–200ms
    // delay before the real window ID is returned.
    let window_below_cursor = {
      let window_id = unsafe {
        CGEvent::integer_value_field(
          Some(cg_event_ref),
          CGEventField::MouseEventWindowUnderMousePointer,
        )
      };

      if window_id == 0 {
        None
      } else {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Some(WindowId(window_id as u32))
      }
    };

    data.pressed_buttons.update(event_kind);

    // Throttle mouse move events so that there's a minimum of 50ms between
    // each emission. State change events (button down/up) always get
    // emitted.
    let should_emit = match event_kind {
      MouseEventKind::Move => {
        let has_elapsed_throttle =
          data.last_move_emission.is_none_or(|timestamp| {
            timestamp.elapsed() >= Duration::from_millis(50)
          });

        // TODO: This is a hack to let through mouse move events when
        // they contain a window ID. macOS sporadically includes the
        // window ID on mouse events.
        has_elapsed_throttle || window_below_cursor.is_some()
      }
      _ => true,
    };

    let mouse_event = match event_kind {
      MouseEventKind::LeftButtonDown => MouseEvent::ButtonDown {
        position,
        button: MouseButton::Left,
        pressed_buttons: data.pressed_buttons,
      },
      MouseEventKind::LeftButtonUp => MouseEvent::ButtonUp {
        position,
        button: MouseButton::Left,
        pressed_buttons: data.pressed_buttons,
      },
      MouseEventKind::RightButtonDown => MouseEvent::ButtonDown {
        position,
        button: MouseButton::Right,
        pressed_buttons: data.pressed_buttons,
      },
      MouseEventKind::RightButtonUp => MouseEvent::ButtonUp {
        position,
        button: MouseButton::Right,
        pressed_buttons: data.pressed_buttons,
      },
      MouseEventKind::Move => MouseEvent::Move {
        position,
        pressed_buttons: data.pressed_buttons,
        window_below_cursor,
      },
    };

    if should_emit {
      let _ = data.event_tx.send(mouse_event);

      if event_kind == MouseEventKind::Move {
        data.last_move_emission = Some(Instant::now());
      }
    }

    unsafe { cg_event.as_mut() }
  }
}

impl Drop for MouseListener {
  fn drop(&mut self) {
    if let Err(err) = self.terminate() {
      tracing::warn!("Failed to terminate mouse listener: {}", err);
    }
  }
}
