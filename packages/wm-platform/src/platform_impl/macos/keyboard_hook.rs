use std::{os::raw::c_void, ptr::NonNull};

use dispatch2::MainThreadBound;
use objc2::MainThreadMarker;
use objc2_core_foundation::{
  kCFRunLoopCommonModes, CFMachPort, CFRetained, CFRunLoop,
};
use objc2_core_graphics::{
  CGEvent, CGEventField, CGEventFlags, CGEventMask, CGEventTapLocation,
  CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
};

use crate::{Dispatcher, Error, Key, KeyCode};

/// macOS-specific keyboard event.
#[derive(Clone, Debug)]
pub struct KeyEvent {
  /// The key that was pressed or released.
  pub key: Key,

  /// Key code that generated this event.
  pub key_code: KeyCode,

  /// Whether the event is for a key press or release.
  pub is_keypress: bool,

  /// Modifier key flags at the time of the event.
  event_flags: CGEventFlags,
}

impl KeyEvent {
  pub(crate) fn new(
    key: Key,
    key_code: KeyCode,
    is_keypress: bool,
    event_flags: CGEventFlags,
  ) -> Self {
    Self {
      key,
      key_code,
      is_keypress,
      event_flags,
    }
  }

  /// Gets whether the specified key is currently pressed.
  pub fn is_key_down(&self, key: Key) -> bool {
    match key {
      Key::Cmd => {
        self.event_flags & CGEventFlags::MaskCommand
          != CGEventFlags::empty()
      }
      Key::Alt => {
        self.event_flags & CGEventFlags::MaskAlternate
          != CGEventFlags::empty()
      }
      Key::Ctrl => {
        self.event_flags & CGEventFlags::MaskControl
          != CGEventFlags::empty()
      }
      Key::Shift => {
        self.event_flags & CGEventFlags::MaskShift != CGEventFlags::empty()
      }
      _ => {
        // TODO: For non-modifier keys, check using CGEventSourceStateID.
        false
      }
    }
  }
}

/// macOS-specific keyboard hook.
#[derive(Debug)]
pub struct KeyboardHook {
  event_tap: Option<MainThreadBound<CFRetained<CFMachPort>>>,
  dispatcher: Dispatcher,
}

impl KeyboardHook {
  /// Creates an instance of `KeyboardHook`.
  ///
  /// The callback is called for every keyboard event and returns
  /// `true` if the event should be intercepted.
  pub fn new<F>(dispatcher: Dispatcher, callback: F) -> crate::Result<Self>
  where
    F: Fn(KeyEvent) -> bool + Send + Sync + 'static,
  {
    let event_tap =
      dispatcher.dispatch_sync(|| Self::create_event_tap(callback))??;

    Ok(Self {
      event_tap: Some(event_tap),
      dispatcher,
    })
  }

  /// Creates a `CGEventTap` object.
  pub fn create_event_tap<F>(
    callback: F,
  ) -> crate::Result<MainThreadBound<CFRetained<CFMachPort>>>
  where
    F: Fn(KeyEvent) -> bool + Send + Sync + 'static,
  {
    let mask: CGEventMask = (1u64 << u64::from(CGEventType::KeyDown.0))
      | (1u64 << u64::from(CGEventType::KeyUp.0));

    // Box the callback and convert to raw pointer for C callback.
    let callback_box = Box::new(callback);
    let callback_ptr = Box::into_raw(callback_box).cast::<c_void>();

    let event_tap = unsafe {
      CGEvent::tap_create(
        CGEventTapLocation::SessionEventTap,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::ListenOnly,
        mask,
        Some(Self::keyboard_event_callback::<F>),
        callback_ptr,
      )
      .ok_or_else(|| {
        // Cleanup callback if tap creation fails.
        let _ = Box::from_raw(callback_ptr.cast::<F>());

        Error::Platform(
          "Failed to create CGEventTap. Accessibility permissions
    may be required."
            .to_string(),
        )
      })
    }?;

    let loop_source =
      CFMachPort::new_run_loop_source(None, Some(&event_tap), 0)
        .ok_or_else(|| {
          Error::Platform("Failed to create loop source".to_string())
        })?;

    let current_loop = CFRunLoop::current().ok_or_else(|| {
      Error::Platform("Failed to get current run loop".to_string())
    })?;

    current_loop
      .add_source(Some(&loop_source), unsafe { kCFRunLoopCommonModes });

    unsafe { CGEvent::tap_enable(&event_tap, true) };

    let event_tap = MainThreadBound::new(event_tap, unsafe {
      MainThreadMarker::new_unchecked()
    });

    Ok(event_tap)
  }

  /// Callback function for keyboard events.
  ///
  /// For use with `CGEventTap`.
  extern "C-unwind" fn keyboard_event_callback<F>(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    mut event: NonNull<CGEvent>,
    user_info: *mut c_void,
  ) -> *mut CGEvent
  where
    F: Fn(KeyEvent) -> bool + Send + Sync + 'static,
  {
    if user_info.is_null() {
      tracing::error!("Null pointer passed to keyboard event callback.");
      return unsafe { event.as_mut() };
    }

    // Extract the key code of the pressed/released key.
    let key_code = KeyCode(unsafe {
      CGEvent::integer_value_field(
        Some(event.as_ref()),
        CGEventField::KeyboardEventKeycode,
      )
    });

    // Try to convert the key code to a known key.
    let Ok(key) = Key::try_from(key_code) else {
      return unsafe { event.as_mut() };
    };

    let is_keypress = event_type == CGEventType::KeyDown;
    let event_flags = unsafe { CGEvent::flags(Some(event.as_ref())) };

    tracing::debug!(
      "Key event: code={}, flags={:?}, is_keypress={}",
      key_code,
      event_flags,
      is_keypress
    );

    let key_event = KeyEvent::new(key, key_code, is_keypress, event_flags);

    // Get callback from user data and invoke it.
    let callback = unsafe { &*(user_info as *const F) };
    let should_intercept = callback(key_event);

    if should_intercept {
      std::ptr::null_mut()
    } else {
      unsafe { event.as_mut() }
    }
  }

  /// Stops the keyboard hook by disabling the `CGEventTap`.
  #[allow(clippy::unnecessary_wraps)]
  pub fn stop(&mut self) -> crate::Result<()> {
    if let Some(tap) = self.event_tap.take() {
      self.dispatcher.dispatch(move || unsafe {
        let tap_ref = tap.get(MainThreadMarker::new_unchecked());
        CGEvent::tap_enable(tap_ref, false);
      })?;
    }

    Ok(())
  }
}

impl Drop for KeyboardHook {
  fn drop(&mut self) {
    let _ = self.stop();
  }
}
